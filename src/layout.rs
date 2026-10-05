use unicode_width::UnicodeWidthStr;
use crate::mem::SystemMemory;
use crate::process::ProcessConsumer;

pub const PALETTE_TOP10: [&str; 10] = [
    "\x1b[38;5;48m",   // 1. Мятный зеленый
    "\x1b[38;5;51m",   // 2. Яркий циан
    "\x1b[38;5;220m",  // 3. Золотистый
    "\x1b[38;5;208m",  // 4. Оранжевый
    "\x1b[38;5;141m",  // 5. Лавандовый
    "\x1b[38;5;205m",  // 6. Розовый
    "\x1b[38;5;119m",  // 7. Салатовый
    "\x1b[38;5;75m",   // 8. Голубой
    "\x1b[38;5;215m",  // 9. Персиковый
    "\x1b[38;5;177m",  // 10. Сиреневый
];

pub const C_OTHER: &str  = "\x1b[38;5;247m";
pub const C_CACHE: &str  = "\x1b[38;5;39m";
pub const C_FREE: &str   = "\x1b[38;5;238m";
pub const C_RESET: &str  = "\x1b[0m";
pub const C_BOLD: &str   = "\x1b[1m";
pub const C_MUTED: &str  = "\x1b[38;5;244m";
pub const C_BORDER: &str = "\x1b[38;5;239m";
pub const SYM: &str      = "■";

#[derive(Clone)]
pub struct LayoutItem {
    pub name: String,
    pub short_name: String,
    pub mem: u64,
    pub color: &'static str,
}

pub fn fmt_size(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} GiB", b / (1024.0 * 1024.0 * 1024.0))
    } else if b >= 1024.0 * 1024.0 {
        format!("{:.1} MiB", b / (1024.0 * 1024.0))
    } else {
        format!("{:.0} KiB", b / 1024.0)
    }
}

pub fn allocate_cells(items: &[LayoutItem], total_cells: usize, total_mem: u64) -> Vec<usize> {
    if total_cells == 0 || total_mem == 0 {
        return vec![0; items.len()];
    }

    let mut floored = Vec::with_capacity(items.len());
    let mut fractions = Vec::with_capacity(items.len());
    let mut sum_allocated = 0;

    for (i, it) in items.iter().enumerate() {
        let exact = (it.mem as f64 / total_mem as f64) * (total_cells as f64);
        let fl = exact.floor() as usize;
        floored.push(fl);
        sum_allocated += fl;
        fractions.push((exact - (fl as f64), i));
    }

    let remainder = total_cells.saturating_sub(sum_allocated);
    fractions.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    for r in 0..remainder {
        let idx = fractions[r % fractions.len()].1;
        floored[idx] += 1;
    }

    floored
}

pub fn render_frame(term_cols: u16, term_rows: u16, sys: &SystemMemory, procs: &[ProcessConsumer]) -> String {
    let total_uni = sys.total_unified;
    let top_mem_sum: u64 = procs.iter().map(|p| p.mem).sum();
    let cache_mem = sys.total_cache;
    let free_mem = sys.free_real;
    let other_mem = total_uni.saturating_sub(top_mem_sum + cache_mem + free_mem);

    let mut items = Vec::new();
    for (i, p) in procs.iter().enumerate() {
        let count_suffix = if p.count > 1 { format!(" (x{})", p.count) } else { String::new() };
        items.push(LayoutItem {
            name: format!("{}{}", p.name, count_suffix),
            short_name: p.short_name.clone(),
            mem: p.mem,
            color: PALETTE_TOP10[i % PALETTE_TOP10.len()],
        });
    }

    items.push(LayoutItem {
        name: "Всё остальное".to_string(),
        short_name: "Всё остальное".to_string(),
        mem: other_mem,
        color: C_OTHER,
    });
    items.push(LayoutItem {
        name: "Суммарный кэш + буферы".to_string(),
        short_name: "Кэш + буферы".to_string(),
        mem: cache_mem,
        color: C_CACHE,
    });
    items.push(LayoutItem {
        name: "Абсолютно свободно".to_string(),
        short_name: "Свободно".to_string(),
        mem: free_mem,
        color: C_FREE,
    });

    let box_w = (term_cols as usize).saturating_sub(2).max(54);
    let inner_w = box_w.saturating_sub(2);

    let t_ram = fmt_size(sys.total_ram);
    let t_ssd = fmt_size(sys.ssd_total);
    let mut title = format!(" ЕДИНОЕ ПРОСТРАНСТВО ПАМЯТИ (RAM {} + SSD SWAP {}) ", t_ram, t_ssd);
    if title.width() > inner_w.saturating_sub(4) {
        title = format!(" ЕДИНАЯ ПАМЯТЬ ({}) ", fmt_size(total_uni));
    }

    let pad_t = inner_w.saturating_sub(title.width());
    let top_border = format!(
        "{C_BORDER}┌─{C_RESET}{C_BOLD}{title}{C_RESET}{C_BORDER}{}┐{C_RESET}",
        "─".repeat(pad_t.saturating_sub(1))
    );

    let two_columns = inner_w >= 84;
    let legend_rows_count = if two_columns { 7 } else { items.len() };
    let fixed_lines = 1 + 1 + 1 + 1 + 1 + legend_rows_count + 1 + 1 + 1;
    let grid_rows = (term_rows as usize).saturating_sub(fixed_lines + 1).max(3);
    let grid_cols = ((inner_w.saturating_sub(4) + 1) / 2).max(8);
    let total_cells = grid_rows * grid_cols;

    let cell_counts = allocate_cells(&items, total_cells, total_uni);

    let mut stream_idx = Vec::with_capacity(total_cells);
    for (it_i, &count) in cell_counts.iter().enumerate() {
        stream_idx.extend(std::iter::repeat(it_i).take(count));
    }
    stream_idx.truncate(total_cells);

    let mut grid_2d = Vec::with_capacity(grid_rows);
    for r in 0..grid_rows {
        let start = r * grid_cols;
        let end = (start + grid_cols).min(stream_idx.len());
        grid_2d.push(&stream_idx[start..end]);
    }

    // Выбор оптимального сегмента для отображения текстовой метки
    let mut runs_by_item: Vec<Vec<(usize, usize, usize, usize)>> = vec![Vec::new(); items.len()];
    for r in 0..grid_rows {
        let mut cur_it: Option<usize> = None;
        let mut start_c = 0;
        for c in 0..grid_cols {
            let it = grid_2d[r][c];
            if Some(it) != cur_it {
                if let Some(prev) = cur_it {
                    runs_by_item[prev].push((r, start_c, c - 1, c - start_c));
                }
                cur_it = Some(it);
                start_c = c;
            }
        }
        if let Some(prev) = cur_it {
            runs_by_item[prev].push((r, start_c, grid_cols - 1, grid_cols - start_c));
        }
    }

    let mut best_runs = vec![None; items.len()];
    for (it_i, runs) in runs_by_item.iter().enumerate() {
        if runs.is_empty() {
            continue;
        }
        let max_len = runs.iter().map(|run| run.3).max().unwrap_or(0);
        let candidates: Vec<&(usize, usize, usize, usize)> = runs.iter().filter(|r| r.3 == max_len).collect();
        let min_r = runs.iter().map(|r| r.0).min().unwrap();
        let max_r = runs.iter().map(|r| r.0).max().unwrap();
        let mid_r = (min_r + max_r) / 2;

        if let Some(chosen) = candidates.iter().min_by_key(|r| (r.0 as isize - mid_r as isize).abs()) {
            best_runs[it_i] = Some((chosen.0, chosen.1, chosen.2));
        }
    }

    let box_line = |content: &str| -> String {
        let v_len = content.width();
        let pad = inner_w.saturating_sub(v_len + 2);
        format!("{C_BORDER}│{C_RESET} {content}{}{C_BORDER}│{C_RESET}", " ".repeat(pad))
    };

    let mut out = String::with_capacity(4096);
    out.push_str(&top_border);
    out.push('\n');

    let sub_line = format!(
        "{C_MUTED}Совокупная память:{C_RESET} {C_BOLD}{}{C_RESET}  {C_MUTED}| RAM: {} | Swap: {}{C_RESET}",
        fmt_size(total_uni), t_ram, t_ssd
    );
    out.push_str(&box_line(&sub_line));
    out.push('\n');
    out.push_str(&format!("{C_BORDER}├{}┤{C_RESET}\n", "─".repeat(inner_w)));

    // Матрица блоков
    let grid_w = grid_cols * 2 - 1;
    let pad_left = (inner_w.saturating_sub(grid_w)) / 2;
    let pad_right = inner_w.saturating_sub(grid_w + pad_left);

    for r in 0..grid_rows {
        let mut row_segments = Vec::new();
        let mut cur_it: Option<usize> = None;
        let mut start_c = 0;

        for c in 0..grid_cols {
            let it = grid_2d[r][c];
            if Some(it) != cur_it {
                if let Some(prev) = cur_it {
                    row_segments.push((prev, start_c, c - 1));
                }
                cur_it = Some(it);
                start_c = c;
            }
        }
        if let Some(prev) = cur_it {
            row_segments.push((prev, start_c, grid_cols - 1));
        }

        let mut seg_rendered = Vec::new();
        for (it_i, s_col, e_col) in row_segments {
            let it = &items[it_i];
            let color = it.color;
            let run_cells = e_col - s_col + 1;
            let w_avail = run_cells * 2 - 1;

            let is_target = best_runs[it_i] == Some((r, s_col, e_col));
            let mut label_chosen = None;

            if is_target && w_avail >= 8 {
                let opt1 = format!("[ {} · {} ]", it.name, fmt_size(it.mem));
                let opt2 = format!("[ {} · {} ]", it.short_name, fmt_size(it.mem));
                let opt3 = format!("[ {} ]", it.short_name);
                let opt4 = it.short_name.clone();

                for opt in [opt1, opt2, opt3, opt4] {
                    if opt.width() + 2 <= w_avail {
                        label_chosen = Some(opt);
                        break;
                    }
                }
            }

            if let Some(label) = label_chosen {
                let l_len = label.width();
                let rem = w_avail - l_len;
                let left_space = rem / 2;
                let right_space = rem - left_space;

                let left_blocks = left_space / 2;
                let left_rem = left_space % 2;
                let l_fill = format!("{}{}", format!("{color}{SYM}{C_RESET} ").repeat(left_blocks), " ".repeat(left_rem));

                let right_blocks = right_space / 2;
                let right_rem = right_space % 2;
                let r_fill = format!("{}{}", " ".repeat(right_rem), format!(" {color}{SYM}{C_RESET}").repeat(right_blocks));

                seg_rendered.push(format!("{l_fill}{C_BOLD}{color}{label}{C_RESET}{r_fill}"));
            } else {
                let blocks: Vec<String> = (0..run_cells).map(|_| format!("{color}{SYM}{C_RESET}")).collect();
                seg_rendered.push(blocks.join(" "));
            }
        }

        let line_content = seg_rendered.join(" ");
        out.push_str(&format!(
            "{C_BORDER}│{C_RESET}{}{line_content}{}{C_BORDER}│{C_RESET}\n",
            " ".repeat(pad_left),
            " ".repeat(pad_right)
        ));
    }

    out.push_str(&format!("{C_BORDER}├{}┤{C_RESET}\n", "─".repeat(inner_w)));
    out.push_str(&box_line(&format!("{C_BOLD}РАСПРЕДЕЛЕНИЕ ОБЪЕДИНЕННОЙ ПАМЯТИ (ТОП ПОТРЕБИТЕЛЕЙ):{C_RESET}")));
    out.push('\n');

    let format_item = |it: Option<&LayoutItem>, width: usize| -> String {
        let it = match it {
            Some(i) => i,
            None => return " ".repeat(width),
        };
        let pct = (it.mem as f64 / total_uni as f64) * 100.0;
        let val_str = fmt_size(it.mem);
        let max_name = width.saturating_sub(21).max(10);
        let name_str = if it.name.width() > max_name {
            let mut s = String::new();
            for ch in it.name.chars() {
                if (s.clone() + &ch.to_string()).width() >= max_name.saturating_sub(1) {
                    break;
                }
                s.push(ch);
            }
            s.push('…');
            s
        } else {
            it.name.clone()
        };

        let left = format!(" {}{SYM}{C_RESET} {C_BOLD}{name_str}{C_RESET}", it.color);
        let right = format!("{val_str:>9} ({pct:>4.1f}%)");
        let sp = width.saturating_sub(left.width() + right.width()).max(1);
        format!("{left}{}{right}", " ".repeat(sp))
    };

    if two_columns {
        let col_w = (inner_w.saturating_sub(3)) / 2;
        let left_items = &items[..items.len().min(7)];
        let right_items = &items[items.len().min(7)..];

        for row_i in 0..legend_rows_count {
            let it_l = left_items.get(row_i);
            let it_r = right_items.get(row_i);
            let txt_l = format_item(it_l, col_w);
            let txt_r = format_item(it_r, col_w);
            let rem_sp = inner_w.saturating_sub(col_w * 2 + 3);
            out.push_str(&format!("{C_BORDER}│{C_RESET} {txt_l} {C_BORDER}│{C_RESET} {txt_r}{}{C_BORDER}│{C_RESET}\n", " ".repeat(rem_sp)));
        }
    } else {
        for it in &items {
            let txt = format_item(Some(it), inner_w.saturating_sub(2));
            out.push_str(&format!("{C_BORDER}│{C_RESET} {txt} {C_BORDER}│{C_RESET}\n"));
        }
    }

    out.push_str(&format!("{C_BORDER}├{}┤{C_RESET}\n", "─".repeat(inner_w)));
    let avail_str = format!(
        "  {C_BOLD}Доступно для запуска новых задач:{C_RESET} {}{C_BOLD}{}{C_RESET}",
        PALETTE_TOP10[0], fmt_size(sys.total_avail)
    );
    out.push_str(&box_line(&avail_str));
    out.push('\n');

    let hint = " [q / Esc - выход] ";
    let pad_b = inner_w.saturating_sub(hint.width() + 2);
    out.push_str(&format!("{C_BORDER}└{}─{C_MUTED}{hint}{C_RESET}{C_BORDER}─┘{C_RESET}", "─".repeat(pad_b)));

    out
}
