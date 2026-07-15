use crate::{
    sim::{base::PresentFeatures, SimilarityMeasure},
    TermId,
};

use super::ic::IcMicaAccessor;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScoringMode {
    Symmetric,
    Asymmetric,
}

#[derive(Clone)]
pub struct Phenomizer<IC> {
    ic: IC,
    mode: ScoringMode,
}

impl<IC> Phenomizer<IC> {
    /// Change the scoring mode to [`ScoringMode::Symmetric`].
    pub fn symmetric(mut self) -> Self {
        self.mode = ScoringMode::Symmetric;
        self
    }

    /// Change the scoring mode to [`ScoringMode::Asymmetric`].
    pub fn asymmetric(mut self) -> Self {
        self.mode = ScoringMode::Asymmetric;
        self
    }
}

impl<IC> Phenomizer<IC>
where
    IC: IcMicaAccessor,
{
    fn one_sided_sim<'a, I, T>(&self, a: I, b: &T) -> f64
    where
        I: Iterator<Item = &'a TermId>,
        T: PresentFeatures,
    {
        let mut sim = 0.;
        let mut n = 0f64;
        for at in a {
            let mut max_ic = 0f64;

            for bt in b.present_features() {
                max_ic = self.ic.get_ic_mica(at, bt).max(max_ic);
            }

            sim = max_ic + n * sim;
            n += 1.;
            sim /= n;
        }

        sim
    }
}

impl<T, IC> SimilarityMeasure<T> for Phenomizer<IC>
where
    T: PresentFeatures,
    IC: IcMicaAccessor,
{
    type Sim = f64;

    fn compute(&self, a: &T, b: &T) -> Self::Sim {
        let a_sim = self.one_sided_sim(a.present_features(), b);
        match self.mode {
            ScoringMode::Asymmetric => a_sim,
            ScoringMode::Symmetric => {
                let b_sim = self.one_sided_sim(b.present_features(), a);
                (a_sim + b_sim) / 2.
            }
        }
    }
}

#[cfg(test)]
mod test_phenomizer {
    use std::collections::HashMap;

    use super::{Phenomizer, ScoringMode};
    use crate::{
        sim::{ic::TermIdPair, SimilarityMeasure},
        TermId,
    };

    fn ic_mica() -> HashMap<TermIdPair, f64> {
        [(["HP:1", "HP:1"], 4.), (["HP:3", "HP:5"], 2.)]
            .into_iter()
            .map(|t| {
                (
                    TermIdPair::from([&t.0[0].parse().unwrap(), &(t.0[1].parse().unwrap())]),
                    t.1,
                )
            })
            .collect()
    }

    #[test]
    fn compute_symmetric() {
        let a = make_terms(&["HP:1", "HP:2", "HP:3"]);
        let b = make_terms(&["HP:1", "HP:5"]);

        let phenomizer = Phenomizer {
            ic: ic_mica(),
            mode: ScoringMode::Symmetric,
        };

        let ab = phenomizer.compute(&a, &b);
        let ba = phenomizer.compute(&b, &a);

        approx::assert_abs_diff_eq!(ab, 2.5);
        approx::assert_abs_diff_eq!(ba, 2.5);
    }

    #[test]
    fn compute_asymmetric() {
        let a = make_terms(&["HP:1", "HP:2", "HP:3"]);
        let b = make_terms(&["HP:1", "HP:5"]);

        let phenomizer = Phenomizer {
            ic: ic_mica(),
            mode: ScoringMode::Asymmetric,
        };

        let ab = phenomizer.compute(&a, &b);
        let ba = phenomizer.compute(&b, &a);

        approx::assert_abs_diff_eq!(ab, 2.);
        approx::assert_abs_diff_eq!(ba, 3.);
    }

    fn make_terms(curies: &[&str]) -> Vec<TermId> {
        curies
            .into_iter()
            .map(|t| t.parse().expect("CURIE should be OK"))
            .collect()
    }
}
