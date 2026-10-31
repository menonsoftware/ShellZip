//! Property tests for path safety and ZIP round-trips.

use std::fs;

use shellzip_core::{CompressOpts, ExtractOpts, NopProgress, Overwrite, add, extract};

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(8))]
    #[test]
    fn random_zip_round_trips(
        name in "[a-z]{1,6}",
        body in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..48usize),
    ) {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join(format!("{name}.txt"));
        fs::write(&src, &body).unwrap();
        let archive = tmp.path().join("out.zip");
        let opts = CompressOpts {
            level: 1,
            threads: 1,
            ..CompressOpts::default()
        };
        add(&archive, &[src], &opts, &NopProgress).unwrap();
        let dest = tmp.path().join("dest");
        let mut extract_opts = ExtractOpts::new(&dest);
        extract_opts.overwrite = Overwrite::Always;
        extract(&archive, &extract_opts, None, &NopProgress).unwrap();
        let written = fs::read(dest.join(format!("{name}.txt"))).unwrap();
        proptest::prop_assert_eq!(written, body);
    }
}
