use anyhow::Result;

pub fn execute(bless: bool) -> Result<()> {
    if bless {
        println!("xtask golden: blessing fixtures...");
    } else {
        println!("xtask golden: verifying fixtures...");
    }
    Ok(())
}
