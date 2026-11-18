use xt_laipath::{Result, calc_eff_lai, calc_lai_ellipse, calc_lai_path, histogram_stats};

fn main() -> Result<()> {
    let paths = vec![0.2, 0.3, 0.5, 0.5, 0.7, 0.9, 1.0];
    let histogram = histogram_stats(paths, 8, 0.0, 1.0)?;
    let gap = 0.3;
    println!("Effective LAI:{:.8}", calc_eff_lai(gap, 0.0, 0.5)?);
    println!(
        "Path distribution LAI:{:.8}",
        calc_lai_path(&histogram, gap, 0.0, 0.5)?
    );
    println!("Ellipsoidal LAI:{:.8}", calc_lai_ellipse(gap, 0.0, 0.5)?);
    Ok(())
}
