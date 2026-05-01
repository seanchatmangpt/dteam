use anyhow::Result;

pub fn execute(_family: String, _value: u8) -> Result<()> {
    println!("xtask explain executed");
    Ok(())
}
