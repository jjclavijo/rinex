#[cfg(test)]
mod test {
    use crate::{epoch, hardware::Antenna, marker::GeodeticMarker, observation::event::{self, Event}, prelude::Rinex, record, writer::BufferedWriter, EpochFlag, Error, GroundPosition};
    use tempfile::NamedTempFile;
    use std::io::Write;
    use std::io::prelude::*;
    use std::fs::File;

    fn parse_string(s: &str) -> Result<Rinex,Error>
    { 
        let mut file = NamedTempFile::new().expect("panic!");
        file.write_all(s.as_bytes())?;
        Rinex::from_file(file.path().to_string_lossy().as_ref())
    }

    #[test]
    fn test_rinex_from_string() {
    
        let r = parse_string(ONE_EPOCH_RX);
        assert!(r.is_ok());
    }

    #[test]
    fn test_add_event() {
    
        let r = parse_string(ONE_EPOCH_RX).expect("");

        let obs_record = r.record;
        let event1 = Event{
            comments: vec![],
            geodetic_marker: Some(GeodeticMarker::default().with_name("MARC")),
            ground_position: Some(GroundPosition::from_geodetic((30.,30.,0.))),
            rcvr_antenna: Some(Antenna::default().with_height(0.123))
        };

        let epoch1 = epoch::parse_in_timescale("2022 01 09 00 00  0.0000000",hifitime::TimeScale::TAI).expect("");

        let mut evt_record = event::Record::new();

        evt_record.insert((epoch1,EpochFlag::NewSiteOccupation),(None,event1.clone()));

        let re = record::Record::ObsEvtRecord(obs_record.as_obs().unwrap().clone(), evt_record);

        // The event should be the first entry of the mixed btreemap
        assert_eq!(re.as_mixed_obs_evt().unwrap().iter().next().unwrap().0, 
                   &(epoch1,EpochFlag::NewSiteOccupation));
    }

    #[test]
    fn test_add_event_and_plot() {
    
        let r = parse_string(ONE_EPOCH_RX).expect("");

        let obs_record = r.record;
        let event1 = Event{
            comments: vec![],
            geodetic_marker: Some(GeodeticMarker::default().with_name("MARC")),
            ground_position: Some(GroundPosition::from_geodetic((30.,30.,0.))),
            rcvr_antenna: Some(Antenna::default().with_height(0.123))
        };

        let epoch1 = epoch::parse_in_timescale("2022 01 09 00 00  0.0000000",hifitime::TimeScale::TAI).expect("");

        let mut evt_record = event::Record::new();

        evt_record.insert((epoch1,EpochFlag::NewSiteOccupation),(None,event1.clone()));

        let re = record::Record::ObsEvtRecord(obs_record.as_obs().unwrap().clone(), evt_record);

        println!("{}",event1);

        let file = NamedTempFile::new().expect("panic!");

        {
            let mut wter = BufferedWriter::new(&file.path().to_string_lossy()).expect("");
            re.to_file(&r.header, &mut wter).expect("");
        }

        let mut file = File::open(file.path()).expect("Unable to open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read the file");

        println!("{}", contents);
        let vline: Vec<&str> = contents.lines().take(1).collect();
        assert!(vline[0].contains("> 2022 01 09 00 00  0.0000000  3  3"))
    }


    const ONE_EPOCH_RX: &'static str = 
"3.04                OBSERVATION DATA    M: MIXED            RINEX VERSION / TYPE
HEADER CHANGED BY EPN CB ON 2022-01-16                      COMMENT
TO BE CONFORM WITH THE INFORMATION IN                       COMMENT
ftp://epncb.oma.be/pub/station/log/alac.log                 COMMENT
                                                            COMMENT
Mdb2Rinex 4.97.35L                      20220110 023216 UTC PGM / RUN BY / DATE
gfzrnx-1.15-8044    HEADER EDIT         20220110 023217 UTC COMMENT
ALAC                                                        MARKER NAME
13433M001                                                   MARKER NUMBER
IGN-E                                                       OBSERVER / AGENCY
1871480             LEICA GR50          4.50/7.710          REC # / TYPE / VERS
10250007            LEIAR25.R3      LEIT                    ANT # / TYPE
        3.0350        0.0000        0.0000                  ANTENNA: DELTA H/E/N
  5009051.3860   -42072.4860  3935057.4820                  APPROX POSITION XYZ
SNR is mapped to RINEX snr flag value [1-9]                 COMMENT
LX:     < 12dBHz -> 1; 12-17dBHz -> 2; 18-23dBHz -> 3       COMMENT
       24-29dBHz -> 4; 30-35dBHz -> 5; 36-41dBHz -> 6       COMMENT
       42-47dBHz -> 7; 48-53dBHz -> 8; >= 54dBHz -> 9       COMMENT
G   12 C1C L1C S1C C2S L2S S2S C2W L2W S2W C5Q L5Q S5Q      SYS / # / OBS TYPES
R   12 C1C L1C S1C C2P L2P S2P C2C L2C S2C C3Q L3Q S3Q      SYS / # / OBS TYPES
E   12 C1C L1C S1C C5Q L5Q S5Q C7Q L7Q S7Q C8Q L8Q S8Q      SYS / # / OBS TYPES
C    9 C2I L2I S2I C6I L6I S6I C7I L7I S7I                  SYS / # / OBS TYPES
DBHZ                                                        SIGNAL STRENGTH UNIT
    30.000                                                  INTERVAL
  2022     1     9     0     0    0.0000000     GPS         TIME OF FIRST OBS
  2022     1     9     0     0    0.0000000     GPS         TIME OF FIRST OBS
     0                                                      RCV CLOCK OFFS APPL
 23 R01  1 R02 -4 R03  5 R04  6 R05  1 R06 -4 R07  5 R08  6 GLONASS SLOT / FRQ #
    R09 -2 R10 -7 R12 -1 R13 -2 R14 -7 R15  0 R16 -1 R17  4 GLONASS SLOT / FRQ #
    R18 -3 R19  3 R20  2 R21  4 R22 -3 R23  3 R24  2        GLONASS SLOT / FRQ #
 C1C  -71.940 C1P  -71.940 C2C  -71.940 C2P  -71.940        GLONASS COD/PHS/BIS
    18    18  2185     7                                    LEAP SECONDS
                                                            END OF HEADER
> 2022 01 09 00 00  0.0000000  0 40
G01  22345079.240   117424213.48008        48.850    22345080.640    91499404.57507        46.950    22345080.580    91499412.56807        44.500    22345078.900    87686944.70808        49.550
G03  25106377.980   131934909.06607        43.000    25106381.840   102806423.66007        42.300    25106381.060   102806435.67506        37.850    25106380.680    98522827.50406        40.650
G08  20374390.760   107068179.92108        52.300    20374392.480    83429765.07708        52.950    20374391.620    83429761.07908        52.250    20374389.640    79953524.82009        54.350
G10  22464836.260   118053596.15308        52.800    22464836.760    91989838.33208        48.900    22464836.820    91989850.33008        52.700    22464834.400    88156942.76908        49.900
G14  25482797.240   133913006.25106        41.050    25482797.040   104347787.95106        40.500    25482797.220   104347792.96406        36.400    25482799.840    99999984.30606        40.750
G16  23776478.000   124946249.83007        47.950                                                    23776476.200    97360704.42006        41.700
G21  21272595.640   111788341.45808        51.750                                                    21272593.160    87107821.58807        47.750
G23  25140204.480   132112746.71607        42.900    25140203.940   102945017.02806        41.600    25140203.860   102945020.00306        38.650    25140206.580    98655682.93206        41.950
G27  20922795.580   109950073.82208        52.800    20922796.360    85675378.31308        52.350    20922795.820    85675400.31408        52.900    20922794.860    82105592.41108        53.800
G32  23629178.880   124172189.66408        50.500    23629180.440    96757557.73707        46.000    23629179.900    96757548.73508        50.450    23629178.100    92725991.31007        44.550
R01  20207718.320   108021892.15508        49.200    20207720.980    84017055.40007        43.700    20207720.080    84017060.40107        44.650
R02  22668665.500   120964368.87508        49.350    22668667.420    94083413.28907        43.000    22668667.400    94083412.28707        43.550
R08  21849258.020   117001804.29807        45.900    21849257.240    91001400.22507        44.650    21849257.080    91001404.23007        45.300
R14  22065079.340   117619290.65008        51.050    22065080.780    91481666.44207        43.000    22065081.000    91481659.45507        43.300
R15  21933521.660   117206098.42208        51.450    21933521.820    91160302.69907        44.050    21933521.860    91160306.70907        44.650
R17  21756851.360   116425337.47208        49.750    21756851.180    90553038.84307        44.300    21756851.020    90553036.84307        45.250
R23  21652002.000   115823681.99308        49.050
R24  19991637.180   106904342.07208        52.350    19991637.180    83147857.34508        48.300    19991637.120    83147836.34708        48.750
E02  27605588.160   145068366.82347        46.050    27605591.780   108330293.52607        42.450    27605590.300   111156293.16907        44.900    27605590.980   109743288.35807        46.100
E03  27452659.340   144264712.86747        47.100    27452663.160   107730169.46707        42.750    27452661.240   110540510.63007        45.650    27452662.060   109135334.55807        46.700
E05  25917950.400   136199763.89748        50.550    25917953.220   101707637.90207        47.200    25917951.880   104360875.56508        50.000    25917952.200   103034247.25208        51.000
E09  25456731.080   133776035.03748        49.000    25456733.060    99897697.40107        43.000    25456731.160   102503726.65507        45.950    25456732.780   101200712.04507        47.050
E15  24609720.200   129325035.70848        52.250    24609722.940    96573945.32308        49.650    24609721.500    99093263.40208        52.400    24609722.100    97833599.38108        53.450
E27  25684091.520   134970811.04448        51.250    25684093.400   100789893.33307        47.550    25684092.500   103419194.60608        50.250    25684092.740   102104548.48908        51.400
E30  25037690.940   131573963.95248        52.550    25037693.080    98253300.85008        49.400    25037691.760   100816426.58208        52.150    25037692.260    99534855.73408        53.250
E36  26949710.960   141621789.02148        48.850    26949714.300   105756599.19607        44.350    26949712.780   108515457.12007        47.500    26949713.620   107136024.67408        48.350
C05  39802007.240   207259525.74207        43.750    39801999.780   168415288.65306        38.250    39802003.380   160266193.83107        42.750
C06  41105304.920   214046142.15206        38.300    41105297.160   173929963.66906        38.800    41105301.900   165514026.82806        39.700
C09  41522025.540   216216077.25706        40.400    41522019.080   175693230.66105        35.100    41522023.060   167191986.32406        40.250
C16  41118108.940   214112788.44806        41.450    41118102.740   173984160.93105        34.200    41118110.520   165565587.75006        39.950
C27  21629730.020   112631742.32308        52.900    21629725.760    91522526.18908        51.350
C28  24379114.440   126948592.69808        50.250    24379110.340   103156148.50407        46.000
C30  23457040.740   122146985.00708        51.400    23457038.460    99254429.47008        48.050
C32  24157809.820   125796125.61808        50.950    24157807.560   102219677.52908        48.350
C33  25258956.540   131530051.46108        48.800    25258964.300   106878983.33107        45.900
C36  25688765.460   133768134.58807        47.000    25688766.240   108697571.70107        43.800
C39  40864936.440   212794461.51007        43.850    40864929.240   172912897.15106        39.150
C41  22775259.440   118596831.78208        52.800    22775261.180    96369669.32008        51.850
C46  25508616.280   132830053.68907        46.850    25508618.780   107935303.92707        43.600
C58  35204636.800   183319907.70407        42.250
> 2022 01 09 00 00  1.0000000  0  0";
}
