use parsett_rust::parse_title;

#[test]
fn test_season_detection() {
    let test_cases = vec![
        ("2 сезон 24 серия.avi", vec![2]),
        ("2-06. Девичья сила.mkv", vec![2]),
        (
            "2. Discovery-Kak_ustroena_Vselennaya.(2.sezon_8.serii.iz.8).2012.XviD.HDTVRip.Krasnodarka",
            vec![2],
        ),
        ("3 сезон", vec![3]),
        ("3Âº Temporada Bob esponja Pt-Br", vec![3]),
        ("4-13 Cursed (HD).m4v", vec![4]),
        ("13-13-13 2013 DVDrip x264 AAC-MiLLENiUM", vec![]),
        ("24 Season 1-8 Complete with Subtitles", vec![
            1, 2, 3, 4, 5, 6, 7, 8,
        ]),
        ("30 M0N3D4S ESP T01XE08.mkv", vec![1]),
        ("Ace of the Diamond: 1st Season", vec![1]),
        ("Ace of the Diamond: 2nd Season", vec![2]),
        ("Adventure Time 10 th season", vec![10]),
        ("All of Us Are Dead . 2022 . S01 EP #1.2.mkv", vec![1]),
        ("Beavis and Butt-Head - 1a. Temporada", vec![1]),
        ("Boondocks, The - Seasons 1 + 2", vec![1, 2]),
        ("breaking.bad.s01e01.720p.bluray.x264-reward", vec![1]),
        (
            "Breaking Bad Complete Season 1 , 2 , 3, 4 ,5 ,1080p HEVC",
            vec![1, 2, 3, 4, 5],
        ),
        ("Bron - S4 - 720P - SweSub.mp4", vec![4]),
        ("clny.3x11m720p.es[www.planetatorrent.com].mkv", vec![3]),
        (
            "Coupling Season 1 - 4 Complete DVDRip - x264 - MKV by RiddlerA",
            vec![1, 2, 3, 4],
        ),
        (
            "DARKER THAN BLACK - S00E04 - Darker Than Black Gaiden OVA 3.mkv",
            vec![0],
        ),
        ("Desperate.Housewives.S0615.400p.WEB-DL.Rus.Eng.avi", vec![
            6,
        ]),
        (
            "Desperate Housewives - Episode 1.22 - Goodbye for now.avi",
            vec![1],
        ),
        (
            "Discovery. Парни с Юкона / Yokon Men [06х01-08] (2017) HDTVRip от GeneralFilm | P1",
            vec![6],
        ),
        ("Doctor.Who.2005.8x11.Dark.Water.720p.HDTV.x264-FoV", vec![
            8,
        ]),
        ("Doctor Who S01--S07--Complete with holiday episodes", vec![
            1, 2, 3, 4, 5, 6, 7,
        ]),
        (
            "Dragon Ball Super S01 E23 French 1080p HDTV H264-Kesni",
            vec![1],
        ),
        ("Dragon Ball [5.134] Preliminary Peril.mp4", vec![5]),
        ("Elementar 3º Temporada Dublado", vec![3]),
        ("Empty Nest Season 1 (1988 - 89) fiveofseven", vec![1]),
        (
            "Eu, a Patroa e as Crianças  4° Temporada Completa - HDTV - Dublado",
            vec![4],
        ),
        (
            "Friends.Complete.Series.S01-S10.720p.BluRay.2CH.x265.HEVC-PSA",
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        ),
        (
            "Friends S04 Season 4 1080p 5.1Ch BluRay ReEnc-DeeJayAhmed",
            vec![4],
        ),
        (
            "Futurama Season 1 2 3 4 5 6 7 + 4 Movies - threesixtyp",
            vec![1, 2, 3, 4, 5, 6, 7],
        ),
        ("Game Of Thrones - Season 1 to 6 (Eng Subs)", vec![
            1, 2, 3, 4, 5, 6,
        ]),
        (
            "Game Of Thrones Complete Season 1,2,3,4,5,6,7 406p mkv + Subs",
            vec![1, 2, 3, 4, 5, 6, 7],
        ),
        (
            "Game of Thrones / Сезон: 1-8 / Серии: 1-73 из 73 [2011-2019, США, BDRip 1080p] MVO (LostFilm)",
            vec![1, 2, 3, 4, 5, 6, 7, 8],
        ),
        ("House MD All Seasons (1-8) 720p Ultra-Compressed", vec![
            1, 2, 3, 4, 5, 6, 7, 8,
        ]),
        (
            "How I Met Your Mother Season 1, 2, 3, 4, 5, & 6 + Extras DVDRip",
            vec![1, 2, 3, 4, 5, 6],
        ),
        (
            "Juego de Tronos - Temp.2 [ALTA DEFINICION 720p][Cap.209][Spanish].mkv",
            vec![2],
        ),
        ("Kyoukai no Rinne (TV) 3rd Season - 23 [1080p]", vec![3]),
        ("Los Simpsons Temp 7 DVDrip Espanol De Espana", vec![7]),
        (
            "Mad Men S02 Season 2 720p 5.1Ch BluRay ReEnc-DeeJayAhmed",
            vec![2],
        ),
        ("MARATHON EPISODES/Orphan Black S3 Eps.05-08.mp4", vec![3]),
        (
            "Mash S10E01b Thats Show Biz Part 2 1080p H.264 (moviesbyrizzo upload).mp4",
            vec![10],
        ),
        ("Merl - Temporada 1", vec![1]),
        ("My Little Pony - A Amizade é Mágica - T02E22.mp4", vec![
            2,
        ]),
        ("My Little Pony FiM - 6.01 - No Second Prances.mkv", vec![6]),
        ("Naruto Shippuden Season 1:11", vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
        ]),
        (
            "Once Upon a Time [S01-07] (2011-2017) WEB-DLRip by Generalfilm",
            vec![1, 2, 3, 4, 5, 6, 7],
        ),
        (
            "One Punch Man 01 - 12 Season 1 Complete [720p] [Eng Subs] [Xerxe:16",
            vec![1],
        ),
        (
            "Orange Is The New Black Season 5 Episodes 1-10 INCOMPLETE (LEAKED)",
            vec![5],
        ),
        (
            "Otchayannie.domochozyaiki.(8.sez.21.ser.iz.23).2012.XviD.HDTVRip.avi",
            vec![8],
        ),
        (
            "Perdidos: Lost: Castellano: Temporadas 1 2 3 4 5 6 (Serie Com",
            vec![1, 2, 3, 4, 5, 6],
        ),
        ("Ranma-12-86.mp4", vec![]),
        ("S011E16.mkv", vec![11]),
        ("Seinfeld S02 Season 2 720p WebRip ReEnc-DeeJayAhmed", vec![
            2,
        ]),
        (
            "Seinfeld Season 2 S02 720p AMZN WEBRip x265 HEVC Complete",
            vec![2],
        ),
        (
            "Seizoen 22 - Zon & Maan Ultra Legendes/afl.18 Je ogen op de bal houden!.mp4",
            vec![22],
        ),
        ("Skam.S01-S02-S03.SweSub.720p.WEB-DL.H264", vec![1, 2, 3]),
        ("Smallville (1x02 Metamorphosis).avi", vec![1]),
        (
            "Sons of Anarchy Sn4 Ep14 HD-TV - To Be, Act 2, By Cool Release",
            vec![4],
        ),
        ("South Park Complete Seasons 1: 11", vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
        ]),
        (
            "Stargate Atlantis ALL Seasons - S01 / S02 / S03 / S04 / S05",
            vec![1, 2, 3, 4, 5],
        ),
        (
            "Stargate Atlantis Complete (Season 1 2 3 4 5) 720p HEVC x265",
            vec![1, 2, 3, 4, 5],
        ),
        ("Teen Titans Season 1-5", vec![1, 2, 3, 4, 5]),
        ("Teen Wolf - 04ª Temporada 720p", vec![4]),
        (
            "The.Man.In.The.High.Castle1x01.HDTV.XviD[www.DivxTotaL.com].avi",
            vec![1],
        ),
        ("The Boondocks Season 1, 2 & 3", vec![1, 2, 3]),
        ("The Boondocks Seasons 1-4 MKV", vec![1, 2, 3, 4]),
        ("The Expanse Complete Seasons 01 & 02 1080p", vec![1, 2]),
        (
            "The Nile Egypts Great River with Bettany Hughes Series 1 4of4 10",
            vec![1],
        ),
        ("The Simpsons S28E21 720p HDTV x264-AVS", vec![28]),
        (
            "The Simpsons Season 20 21 22 23 24 25 26 27 - threesixtyp",
            vec![20, 21, 22, 23, 24, 25, 26, 27],
        ),
        ("The Twilight Zone 1985 S01E22c The Library.mp4", vec![1]),
        ("The Twilight Zone 1985 S01E23a Shadow Play.mp4", vec![1]),
        (
            "The Walking Dead [Temporadas 1 & 2 Completas Em HDTV E Legena",
            vec![1, 2],
        ),
        (
            "Tokyo Ghoul Root A - 07 [S2-07] [Eng Sub] 480p [email protected]",
            vec![2],
        ),
        ("Travelers - Seasons 1 and 2 - Mp4 x264 AC3 1080p", vec![
            1, 2,
        ]),
        (
            "True Blood Season 1, 2, 3, 4, 5 & 6 + Extras BDRip TSV",
            vec![1, 2, 3, 4, 5, 6],
        ),
        ("Vikings 3 Temporada 720p", vec![3]),
        (
            "Zvezdnie.Voiny.Voina.Klonov.3.sezon.22.seria.iz.22.XviD.HDRip.avi",
            vec![3],
        ),
        ("[5.01] Weight Loss.avi", vec![5]),
        (
            "[Erai-raws] Granblue Fantasy The Animation Season 2 - 08 [1080p][Multiple Subtitle].mkv",
            vec![2],
        ),
        (
            "[Erai-raws] Granblue Fantasy The Animation Season 2 - 10 [1080p][Multiple Subtitle].mkv",
            vec![2],
        ),
        (
            "[Erai-raws] Shingeki no Kyojin Season 3 - 11 (BD 1080p Hi10 FLAC) [1FA13150].mkv",
            vec![3],
        ),
        (
            "[F-D] Fairy Tail Season 1 -6 + Extras [480P][Dual-Audio]",
            vec![1, 2, 3, 4, 5, 6],
        ),
        (
            "[FFA] Kiratto Pri☆chan Season 3 - 11 [1080p][HEVC].mkv",
            vec![3],
        ),
        (
            "[HR] Boku no Hero Academia 87 (S4-24) [1080p HEVC Multi-Subs] HR-GZ",
            vec![4],
        ),
        (
            "[SCY] Attack on Titan Season 3 - 11 (BD 1080p Hi10 FLAC) [1FA13150].mkv",
            vec![3],
        ),
        ("Доктор Хаус 03-20.mkv", vec![3]),
        (
            "Друзья / Friends / Сезон: 1 / Серии: 1-24 из 24 [1994-1995, США, BDRip 720p] MVO + Original + Sub (Rus, Eng)",
            vec![1],
        ),
        (
            "Друзья / Friends / Сезон: 1, 2 / Серии: 1-24 из 24 [1994-1999, США, BDRip 720p] MVO",
            vec![1, 2],
        ),
        ("Интерны. Сезон №9. Серия №180.avi", vec![
            9,
        ]),
        ("Комиссар Рекс 11-13.avi", vec![11]),
        (
            "Леди Баг и Супер-Кот – Сезон 3, Эпизод 21 – Кукловод 2 [1080p].mkv",
            vec![3],
        ),
        (
            "Проклятие острова ОУК_ 5-й сезон 09-я серия_ Прорыв Дэна.avi",
            vec![5],
        ),
        (
            "Разрушители легенд. MythBusters. Сезон 15. Эпизод 09. Скрытая угроза (2015).avi",
            vec![15],
        ),
        ("Сезон 5/Серия 11.mkv", vec![5]),
        ("Vikkatakavi 01E06.mkv", vec![1]),
    ];

    for (release_name, expected_seasons) in test_cases {
        let result = parse_title(release_name).unwrap();
        assert_eq!(
            result.seasons, expected_seasons,
            "Failed for {}",
            release_name
        );
    }
}
