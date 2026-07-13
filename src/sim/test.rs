use crate::{
    sim::base::{ObservationStatus, PresentFeatures},
    TermId,
};

pub(crate) struct Individual {
    label: &'static str,
    features: Vec<TermId>,
    states: Vec<ObservationStatus>,
}

impl Individual {
    pub fn label(&self) -> &'static str {
        self.label
    }

    fn make_sample(label: &'static str, phenotypes: &[(&str, bool)]) -> Self {
        let mut features = vec![];
        let mut states = vec![];
        for ele in phenotypes {
            features.push(ele.0.parse::<TermId>().expect("Curie should be parsable"));
            states.push(if ele.1 {
                ObservationStatus::Present
            } else {
                ObservationStatus::Excluded
            });
        }
        Self {
            label,
            features,
            states,
        }
    }
}

impl PresentFeatures for Individual {
    fn present_features(&self) -> impl Iterator<Item = &TermId> {
        (0..self.features.len())
            .filter(|&i| self.states[i].is_present())
            .map(|i| &self.features[i])
    }
}

pub mod fbn1 {
    use std::{fs::File, io::BufReader};

    use flate2::bufread::GzDecoder;

    use crate::{
        io::OntologyLoaderBuilder,
        ontology::{csr::MinimalCsrOntology, OntologyTerms},
        term::MinimalTerm,
    };

    use super::Individual;

    pub fn bm() -> Individual {
        Individual::make_sample(
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

    pub fn jl() -> Individual {
        Individual::make_sample(
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

    pub fn op() -> Individual {
        Individual::make_sample(
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

    pub fn rwt() -> Individual {
        Individual::make_sample(
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

    pub fn vw() -> Individual {
        Individual::make_sample(
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

        for (term_id, status) in individual.features.iter().zip(individual.states.iter()) {
            let pti = hpo.primary_term_id(term_id).unwrap();
            let term = hpo.term_by_id(pti).expect("Term should be present");
            println!(
                "(\"{}\", {}), // {}",
                term_id,
                status.is_present(),
                term.name()
            );
        }

        Ok(())
    }
}
