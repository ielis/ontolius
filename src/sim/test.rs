use crate::sim::{base::ObservationStatus, feature::IndividualFeature};

pub struct TestIndividual<'a> {
    pub label: &'a str,
    pub features: Vec<IndividualFeature<'a>>,
}

impl<'a> TestIndividual<'a> {
    pub fn new(label: &'a str, features: &'a [(&'a str, bool)]) -> Self {
        let features: Vec<_> = features
            .into_iter()
            .map(|f| {
                IndividualFeature::builder()
                    .owned(f.0.parse().expect("Kosher curie"))
                    .with_status(if f.1 {
                        ObservationStatus::Present
                    } else {
                        ObservationStatus::Excluded
                    })
                    .build()
            })
            .collect();

        Self { label, features }
    }
}

pub mod fbn1 {
    use std::{fs::File, io::BufReader};

    use flate2::bufread::GzDecoder;

    use crate::{
        io::OntologyLoaderBuilder,
        ontology::{csr::MinimalCsrOntology, OntologyTerms},
        sim::Observed,
        term::MinimalTerm,
        Identified,
    };

    use super::TestIndividual;

    pub fn fbn1_individuals() -> Vec<TestIndividual<'static>> {
        vec![bm(), jl(), op(), rwt(), vw()]
    }

    pub fn bm() -> TestIndividual<'static> {
        TestIndividual::new(
            "BM",
            &[
                ("HP:0001083", true),  // Ectopia lentis
                ("HP:0001065", true),  // Striae distensae
                ("HP:0012773", true),  // Reduced upper to lower segment ratio
                ("HP:0000501", false), // Glaucoma
                ("HP:0000545", false), // Myopia
                ("HP:0000486", false), // Strabismus
                ("HP:0002650", false), // Scoliosis
                ("HP:0001382", false), // Joint hypermobility
                ("HP:0000767", false), // Pectus excavatum
                ("HP:0001166", false), // Arachnodactyly
                ("HP:0000541", false), // Retinal detachment
                ("HP:0000768", false), // Pectus carinatum
                ("HP:0000218", false), // High palate
                ("HP:0002616", false), // Aortic root aneurysm
                ("HP:0001634", false), // Mitral valve prolapse
            ],
        )
    }

    pub fn jl() -> TestIndividual<'static> {
        TestIndividual::new(
            "JL",
            &[
                ("HP:0001083", true),  // Ectopia lentis
                ("HP:0000545", true),  // Myopia
                ("HP:0001166", true),  // Arachnodactyly
                ("HP:0000218", true),  // High palate
                ("HP:0001634", true),  // Mitral valve prolapse
                ("HP:0012773", true),  // Reduced upper to lower segment ratio
                ("HP:0000501", false), // Glaucoma
                ("HP:0000486", false), // Strabismus
                ("HP:0002650", false), // Scoliosis
                ("HP:0001382", false), // Joint hypermobility
                ("HP:0000767", false), // Pectus excavatum
                ("HP:0000541", false), // Retinal detachment
                ("HP:0000768", false), // Pectus carinatum
                ("HP:0001065", false), // Striae distensae
                ("HP:0002616", false), // Aortic root aneurysm
            ],
        )
    }

    pub fn op() -> TestIndividual<'static> {
        TestIndividual::new(
            "OP",
            &[
                ("HP:0001083", true),  // Ectopia lentis
                ("HP:0000545", true),  // Myopia
                ("HP:0001166", true),  // Arachnodactyly
                ("HP:0000218", true),  // High palate
                ("HP:0001634", true),  // Mitral valve prolapse
                ("HP:0012773", true),  // Reduced upper to lower segment ratio
                ("HP:0000501", false), // Glaucoma
                ("HP:0000486", false), // Strabismus
                ("HP:0002650", false), // Scoliosis
                ("HP:0001382", false), // Joint hypermobility
                ("HP:0000767", false), // Pectus excavatum
                ("HP:0000541", false), // Retinal detachment
                ("HP:0000768", false), // Pectus carinatum
                ("HP:0001065", false), // Striae distensae
                ("HP:0002616", false), // Aortic root aneurysm
            ],
        )
    }

    pub fn rwt() -> TestIndividual<'static> {
        TestIndividual::new(
            "RWT",
            &[
                ("HP:0001083", true),  // Ectopia lentis
                ("HP:0000545", true),  // Myopia
                ("HP:0000486", true),  // Strabismus
                ("HP:0001382", true),  // Joint hypermobility
                ("HP:0001065", true),  // Striae distensae
                ("HP:0000501", false), // Glaucoma
                ("HP:0002650", false), // Scoliosis
                ("HP:0000767", false), // Pectus excavatum
                ("HP:0001166", false), // Arachnodactyly
                ("HP:0000541", false), // Retinal detachment
                ("HP:0000768", false), // Pectus carinatum
                ("HP:0000218", false), // High palate
                ("HP:0002616", false), // Aortic root aneurysm
                ("HP:0001634", false), // Mitral valve prolapse
                ("HP:0012773", false), // Reduced upper to lower segment ratio
            ],
        )
    }

    pub fn vw() -> TestIndividual<'static> {
        TestIndividual::new(
            "VW",
            &[
                ("HP:0001083", true),  // Ectopia lentis
                ("HP:0000501", true),  // Glaucoma
                ("HP:0002650", true),  // Scoliosis
                ("HP:0000218", true),  // High palate
                ("HP:0001065", true),  // Striae distensae
                ("HP:0000545", false), // Myopia
                ("HP:0000486", false), // Strabismus
                ("HP:0001382", false), // Joint hypermobility
                ("HP:0000767", false), // Pectus excavatum
                ("HP:0001166", false), // Arachnodactyly
                ("HP:0000541", false), // Retinal detachment
                ("HP:0000768", false), // Pectus carinatum
                ("HP:0002616", false), // Aortic root aneurysm
                ("HP:0001634", false), // Mitral valve prolapse
                ("HP:0012773", false), // Reduced upper to lower segment ratio
            ],
        )
    }

    #[test]
    #[ignore = "Run manually on demand"]
    fn print_labels() -> Result<(), Box<dyn std::error::Error>> {
        let builder = OntologyLoaderBuilder::new().obographs_parser().build();

        let mut read = GzDecoder::new(BufReader::new(File::open(
            "resources/hp.v2024-08-13.json.gz",
        )?));
        let hpo: MinimalCsrOntology = builder.load_from_read(&mut read).expect("OK loading");

        let individual = vw();

        for feature in individual.features.iter() {
            let pti = hpo.primary_term_id(feature).unwrap();
            let term = hpo.term_by_id(pti).expect("Term should be present");
            println!(
                "(\"{}\", {}), // {}",
                feature.identifier(),
                feature.is_present(),
                term.name()
            );
        }

        Ok(())
    }
}
