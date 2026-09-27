use mote::config::Manifest;
use mote::model::ModelChain;

fn main() {
    let result = (|| -> Result<(), String> {
        let path = std::env::args()
            .nth(1)
            .ok_or("usage: mote-rust-starter <agent.yaml>")?;
        let yaml = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mut manifest = Manifest::from_yaml(&yaml).map_err(|e| e.to_string())?;
        manifest.workspace = manifest.workspace_from(std::path::Path::new(&path));
        let model = ModelChain::from_manifest(&manifest)?;
        mote_rust_starter::run(&manifest, &model)
    })();
    match result {
        Ok(()) => println!("Verified report.txt: MOTE WORKS"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
