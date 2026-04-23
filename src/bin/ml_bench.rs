/// ML Module Benchmark — timing and accuracy for each classifier
/// Runs against first 3 PDC logs to get representative timing data

use dteam::conformance::bitmask_replay::{classify_exact, in_language, replay_log, NetBitmask64};
use dteam::io::pnml::read_pnml;
use dteam::io::xes::XESReader;
use dteam::ml::pdc_features::extract_log_features;
use dteam::ml::pdc_supervised::run_supervised;
use dteam::ml::pdc_unsupervised::run_unsupervised;
use dteam::models::AttributeValue;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .init();

    let test_dir = PathBuf::from("data/pdc2025/test_logs");
    let model_dir = PathBuf::from("data/pdc2025/models");
    let gt_dir = PathBuf::from("data/pdc2025/ground_truth");

    // Collect first 3 logs
    let mut entries: Vec<_> = std::fs::read_dir(&test_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "xes").unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    entries.truncate(3);

    let reader = XESReader::new();

    println!("\n╔════════════════════════════════════════════════════════════════════╗");
    println!("║              ML MODULE BENCHMARK — PDC 2025                        ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");

    let mut results: Vec<BenchResult> = Vec::new();

    for entry in &entries {
        let log_path = entry.path();
        let stem = log_path.file_stem().unwrap().to_string_lossy().into_owned();

        let gt_path = gt_dir.join(format!("{}.xes", stem));
        let model_path = model_dir.join(format!("{}.pnml", stem));

        let log = match reader.read(&log_path) {
            Ok(l) => l,
            Err(_) => continue,
        };
        let gt = match reader.read(&gt_path) {
            Ok(l) => l,
            Err(_) => continue,
        };

        let labels_gt: Vec<bool> = gt
            .traces
            .iter()
            .map(|t| {
                t.attributes
                    .iter()
                    .find(|a| a.key == "pdc:isPos")
                    .and_then(|a| {
                        if let AttributeValue::Boolean(b) = &a.value {
                            Some(*b)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(false)
            })
            .collect();

        if model_path.exists() {
            if let Ok(dnet) = read_pnml(&model_path) {
                if dnet.places.len() <= 64 {
                    let bm = NetBitmask64::from_petri_net(&dnet);

                    println!("Log: {} ({} traces, {} places)", stem, log.traces.len(), dnet.places.len());
                    println!("├─ Conformance Strategies");

                    // F: classify_exact
                    let t0 = Instant::now();
                    let f = classify_exact(&bm, &log, 500);
                    let tf = t0.elapsed();
                    let acc_f = f.iter().zip(&labels_gt).filter(|(p, &gt)| **p == gt).count() as f64
                        / labels_gt.len() as f64;
                    results.push(BenchResult {
                        log: stem.clone(),
                        module: "classify_exact (F)".to_string(),
                        timing_us: tf.as_micros() as u64,
                        accuracy: acc_f,
                        category: "Conformance".to_string(),
                    });
                    println!(
                        "│  ├─ classify_exact (F):      {:>8.2} ms,  {:.2}% acc",
                        tf.as_secs_f64() * 1000.0,
                        acc_f * 100.0
                    );

                    // G: fitness only
                    let t0 = Instant::now();
                    let results_g = replay_log(&bm, &log);
                    let g: Vec<bool> = {
                        let mut ranked: Vec<(usize, f64)> = results_g
                            .iter()
                            .enumerate()
                            .map(|(i, r)| (i, r.fitness()))
                            .collect();
                        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
                        let mut out = vec![false; log.traces.len()];
                        for &(i, _) in ranked.iter().take(500) {
                            out[i] = true;
                        }
                        out
                    };
                    let tg = t0.elapsed();
                    let acc_g = g.iter().zip(&labels_gt).filter(|(p, &gt)| **p == gt).count() as f64
                        / labels_gt.len() as f64;
                    results.push(BenchResult {
                        log: stem.clone(),
                        module: "fitness_rank (G)".to_string(),
                        timing_us: tg.as_micros() as u64,
                        accuracy: acc_g,
                        category: "Conformance".to_string(),
                    });
                    println!(
                        "│  ├─ fitness_rank (G):        {:>8.2} ms,  {:.2}% acc",
                        tg.as_secs_f64() * 1000.0,
                        acc_g * 100.0
                    );

                    // H: in_language + fitness fill
                    let t0 = Instant::now();
                    let in_lang: Vec<bool> = log.traces.iter().map(|t| in_language(&bm, t)).collect();
                    let h = {
                        let n_clean = in_lang.iter().filter(|&&b| b).count();
                        if n_clean >= 500 {
                            in_lang.clone()
                        } else {
                            let mut sorted_remaining: Vec<(usize, f64)> = results_g
                                .iter()
                                .enumerate()
                                .filter(|(i, _)| !in_lang[*i])
                                .map(|(i, r)| (i, r.fitness()))
                                .collect();
                            sorted_remaining.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                            let mut out = in_lang.clone();
                            let fill = 500usize.saturating_sub(n_clean);
                            for &(i, _) in sorted_remaining.iter().take(fill) {
                                out[i] = true;
                            }
                            out
                        }
                    };
                    let th = t0.elapsed();
                    let acc_h = h.iter().zip(&labels_gt).filter(|(p, &gt)| **p == gt).count() as f64
                        / labels_gt.len() as f64;
                    results.push(BenchResult {
                        log: stem.clone(),
                        module: "in_language+fill (H)".to_string(),
                        timing_us: th.as_micros() as u64,
                        accuracy: acc_h,
                        category: "Conformance".to_string(),
                    });
                    println!(
                        "│  └─ in_language+fill (H):    {:>8.2} ms,  {:.2}% acc",
                        th.as_secs_f64() * 1000.0,
                        acc_h * 100.0
                    );

                    println!("├─ ML Strategies");

                    // Feature extraction
                    let t0 = Instant::now();
                    let (features, in_lang_flags, fitness) = extract_log_features(&log, &bm);
                    let t_feat = t0.elapsed();
                    results.push(BenchResult {
                        log: stem.clone(),
                        module: "extract_log_features".to_string(),
                        timing_us: t_feat.as_micros() as u64,
                        accuracy: 0.0,
                        category: "Features".to_string(),
                    });
                    println!(
                        "│  ├─ extract_log_features:   {:>8.2} ms",
                        t_feat.as_secs_f64() * 1000.0
                    );

                    // Supervised
                    let t0 = Instant::now();
                    let sup = run_supervised(&features, &in_lang_flags);
                    let t_sup = t0.elapsed();
                    results.push(BenchResult {
                        log: stem.clone(),
                        module: "run_supervised (11 clf)".to_string(),
                        timing_us: t_sup.as_micros() as u64,
                        accuracy: 0.0,
                        category: "Supervised".to_string(),
                    });
                    println!(
                        "│  ├─ run_supervised (11):     {:>8.2} ms",
                        t_sup.as_secs_f64() * 1000.0
                    );

                    // Unsupervised
                    let t0 = Instant::now();
                    let unsup = run_unsupervised(&features, &[].to_vec().iter().map(|_| None).collect::<Vec<_>>(), &fitness, 500);
                    let t_unsup = t0.elapsed();
                    results.push(BenchResult {
                        log: stem.clone(),
                        module: "run_unsupervised (5 clf)".to_string(),
                        timing_us: t_unsup.as_micros() as u64,
                        accuracy: 0.0,
                        category: "Unsupervised".to_string(),
                    });
                    println!(
                        "│  └─ run_unsupervised (5):    {:>8.2} ms",
                        t_unsup.as_secs_f64() * 1000.0
                    );

                    println!();
                }
            }
        }
    }

    // Summary table
    println!("\n╔════════════════════════════════════════════════════════════════════╗");
    println!("║                      SUMMARY TABLE                                 ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");

    println!("| Module                      | Log        | Time (μs)  | Accuracy |");
    println!("|:----------------------------|:-----------|:-----------|:---------|");

    for r in &results {
        if r.accuracy > 0.0 {
            println!(
                "| {:<27} | {:<10} | {:>10} | {:>7.2}% |",
                r.module, r.log, r.timing_us, r.accuracy * 100.0
            );
        } else {
            println!(
                "| {:<27} | {:<10} | {:>10} |    —     |",
                r.module, r.log, r.timing_us
            );
        }
    }

    println!("\n╔════════════════════════════════════════════════════════════════════╗");
    println!("║                  TIMING SUMMARY (avg across logs)                  ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");

    let mut category_timings: std::collections::HashMap<String, (u64, usize)> =
        std::collections::HashMap::new();
    for r in &results {
        let entry = category_timings.entry(r.category.clone()).or_insert((0, 0));
        entry.0 += r.timing_us;
        entry.1 += 1;
    }

    let mut sorted: Vec<_> = category_timings.into_iter().collect();
    sorted.sort_by_key(|a| a.1 .0);

    for (cat, (total, count)) in sorted {
        let avg = total / count as u64;
        println!("{:<30} {:.2} ms avg ({} runs)", cat, avg as f64 / 1000.0, count);
    }

    println!();
}

#[derive(Debug, Clone)]
struct BenchResult {
    log: String,
    module: String,
    timing_us: u64,
    accuracy: f64,
    category: String,
}
