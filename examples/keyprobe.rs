//! keyprobe — manual AACS key-resolution diagnostic for an ISO.
//!
//! Bypasses the CLI verdict: scans the ISO, reads real encrypted samples,
//! pulls the keydb entry, and INDEPENDENTLY tests whether the keydb's UK/VUK
//! actually decrypt the on-disc units — then reproduces the real resolve path
//! so we can see exactly where selection/validation diverges.
//!
//!   keyprobe <iso://path|/path.iso> <keydb.cfg>

use libfreemkv::aacs::{ALIGNED_UNIT_LEN, decrypt_unit, decrypt_unit_full, is_aacs_scrambled};

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: keyprobe <iso://path|/path.iso> <keydb.cfg>");
        std::process::exit(2);
    }
    let iso = &args[1];
    let keydb_path = &args[2];

    let path = match libfreemkv::parse_url(iso) {
        libfreemkv::StreamUrl::Iso { path } => path,
        _ => std::path::PathBuf::from(iso),
    };

    let mut reader = libfreemkv::FileSectorSource::open(&path).expect("open iso");
    let cap = <libfreemkv::FileSectorSource as libfreemkv::SectorSource>::capacity_sectors(&reader);
    let mut disc =
        libfreemkv::Disc::scan_image(&mut reader, cap, &libfreemkv::ScanOptions::default())
            .expect("scan_image");

    println!("== disc ==");
    println!(
        "format={:?} encrypted={} aacs={}",
        disc.format,
        disc.encrypted,
        disc.aacs.is_some()
    );
    if let Some(a) = &disc.aacs {
        println!(
            "aacs.version={} bus_encryption={} disc_hash={} vuk={:?} read_data_key={:?} unit_keys={}",
            a.version,
            a.bus_encryption,
            a.disc_hash,
            a.vuk.map(|v| hex(&v)),
            a.read_data_key.map(|k| hex(&k)),
            a.unit_keys.len(),
        );
    }
    let inputs = match disc.inputs() {
        Some(i) => i,
        None => {
            println!("NO AACS INPUTS — disc not AACS-encrypted?");
            return;
        }
    };
    println!(
        "inputs.disc_hash={} uk_ro_len={} mkb_len={} vid={}",
        inputs.disc_hash,
        inputs.unit_key_ro.len(),
        inputs.mkb.len(),
        hex(&inputs.volume_id)
    );

    // largest title → samples (same as resolve_iso_unit_keys)
    let title = disc
        .titles
        .iter()
        .max_by_key(|t| t.size_bytes)
        .cloned()
        .expect("a title");
    println!("\n== samples ==");
    println!(
        "largest title playlist={} size={} extents={}",
        title.playlist,
        title.size_bytes,
        title.extents.len()
    );
    let samples = freemkv_keysources::read_sample_units(&mut reader, &title, 8);
    let scrambled: Vec<Vec<u8>> = samples
        .into_iter()
        .filter(|s| s.len() >= ALIGNED_UNIT_LEN && is_aacs_scrambled(s))
        .collect();
    println!("scrambled samples collected: {}", scrambled.len());
    if let Some(s) = scrambled.first() {
        println!("first scrambled sample head[0..16]={}", hex(&s[..16]));
    }

    // keydb entry
    println!("\n== keydb entry ==");
    let db = libfreemkv::aacs::KeyDb::load(std::path::Path::new(keydb_path)).expect("load keydb");
    match db.find_disc(&inputs.disc_hash) {
        None => println!("NO keydb entry for {}", inputs.disc_hash),
        Some(e) => {
            println!("vuk={:?}", e.vuk.map(|v| hex(&v)));
            println!(
                "unit_keys={:?}",
                e.unit_keys
                    .iter()
                    .map(|(n, k)| format!("{n}-{}", hex(k)))
                    .collect::<Vec<_>>()
            );

            // INDEPENDENT TEST: does each keydb UK decrypt a scrambled sample?
            println!("\n== manual decrypt with keydb UK(s) ==");
            for (n, uk) in &e.unit_keys {
                let mut du = 0;
                let mut duf = 0;
                for s in &scrambled {
                    let mut p = s[..ALIGNED_UNIT_LEN].to_vec();
                    if decrypt_unit(&mut p, uk) {
                        du += 1;
                    }
                    let mut p2 = s[..ALIGNED_UNIT_LEN].to_vec();
                    if decrypt_unit_full(&mut p2, uk, None) {
                        duf += 1;
                    }
                }
                println!(
                    "UK cps={n} {}: decrypt_unit {}/{}  decrypt_unit_full(rdk=None) {}/{}",
                    hex(uk),
                    du,
                    scrambled.len(),
                    duf,
                    scrambled.len()
                );
            }
        }
    }

    // REAL PATH: reproduce resolve_and_apply exactly and report the outcome.
    println!("\n== real resolve path (resolve_and_apply via KeydbSource) ==");
    let mut inputs2 = disc.inputs().expect("inputs");
    inputs2.samples = scrambled.clone();
    let sources: Vec<Box<dyn freemkv_keysources::KeySource>> = vec![Box::new(
        freemkv_keysources::KeydbSource::new(std::path::PathBuf::from(keydb_path)),
    )];
    let mut multi = freemkv_keysources::MultiSource::new(sources);
    let applied = freemkv_keysources::resolve_and_apply(&mut multi, &inputs2, &mut disc);
    println!("resolve_and_apply returned: {applied}");
    match disc.decrypt_keys() {
        libfreemkv::DecryptKeys::Aacs {
            unit_keys,
            read_data_key,
        } => {
            println!(
                "decrypt_keys = Aacs {{ unit_keys: {:?}, rdk: {:?} }}",
                unit_keys
                    .iter()
                    .map(|(n, k)| format!("{n}-{}", hex(k)))
                    .collect::<Vec<_>>(),
                read_data_key.map(|k| hex(&k))
            );
        }
        libfreemkv::DecryptKeys::Css { .. } => println!("decrypt_keys = Css"),
        libfreemkv::DecryptKeys::None => {
            println!("decrypt_keys = None  <-- this is the E7022 path")
        }
    }
}
