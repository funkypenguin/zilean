use parsett_rust::parse_title;

#[test]
fn test_resolution_detection() {
    let test_cases = vec![
        ("Annabelle.2014.1080p.PROPER.HC.WEBRip.x264.AAC.2.0-RARBG", "1080p"),
        ("doctor_who_2005.8x12.death_in_heaven.720p_hdtv_x264-fov", "720p"),
        ("UFC 187 PPV 720P HDTV X264-KYR", "720p"),
        ("The Smurfs 2 2013 COMPLETE FULL BLURAY UHD (4K) - IPT EXCLUSIVE", "2160p"),
        ("Joker.2019.2160p.4K.BluRay.x265.10bit.HDR.AAC5.1", "2160p"),
        (
            "[Beatrice-Raws] Evangelion 3.333 You Can (Not) Redo [BDRip 3840x1632 HEVC TrueHD]",
            "2160p",
        ),
        (
            "[Erai-raws] Evangelion 3.0 You Can (Not) Redo - Movie [1920x960][Multiple Subtitle].mkv",
            "1080p",
        ),
        ("[JacobSwaggedUp] Kizumonogatari I: Tekketsu-hen (BD 1280x544) [MP4 Movie]", "720p"),
        ("UFC 187 PPV 720i HDTV X264-KYR", "720p"),
        (
            "IT Chapter Two.2019.7200p.AMZN WEB-DL.H264.[Eng Hin Tam Tel]DDP 5.1.MSubs.D0T.Telly",
            "720p",
        ),
        ("Dumbo (1941) BRRip XvidHD 10800p-NPW", "1080p"),
        ("The Boys S04E01 E02 E03 4k to 1080p AMZN WEBrip x265 DDP5 1 D0c", "1080p"),
        ("Batman Returns 1992 4K Remastered BluRay 1080p DTS AC3 x264-MgB", "1080p"),
        ("Life After People (2008) [1080P.BLURAY] [720p] [BluRay] [YTS.MX]", "720p"),
        ("Life After People (2008) [567P.BLURAY] [567p] [BluRay] [YTS.MX]", "567p"),
    ];

    for (release_name, expected_resolution) in test_cases {
        let result = parse_title(release_name).unwrap();
        assert_eq!(
            result.resolution.as_deref(),
            Some(expected_resolution),
            "Expected resolution to be {} for {}",
            expected_resolution,
            release_name
        );
    }
}
