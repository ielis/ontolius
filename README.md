# Ontolius

A fast and safe crate for working with biomedical ontologies.

## Examples

We provide examples of loading ontology and its subsequent usage
in applications.

### Load ontology 🪄

`ontolius` can load ontology from Obographs JSON file.
For the sake of this example, we use
[flate2](https://github.com/rust-lang/flate2-rs)
to decompress a gzipped JSON on the fly.

We can load a toy version of HPO from a JSON file as follows:

```rust
use std::fs::File;
use std::io::BufReader;

use flate2::bufread::GzDecoder;
use ontolius::io::OntologyLoaderBuilder;
use ontolius::ontology::csr::MinimalCsrOntology;

// Load a toy Obographs file from the repo
let path = "resources/hp.small.json.gz";

// Configure the loader to parse the input as an Obographs file
let loader = OntologyLoaderBuilder::new()
                .obographs_parser()
                .build();

let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
let hpo: MinimalCsrOntology = loader.load_from_read(reader)
                                .expect("HPO should be loaded");
```

We loaded HPO from a toy JSON file into [`crate::ontology::csr::MinimalCsrOntology`].
The loading includes parsing terms and edges from the Obographs file
and construction of the ontology graph.
In case of `MinimalCsrOntology`,
the graph is backed by a compressed sparse row (CSR) adjacency matrix.

See [`crate::io::OntologyLoader`] for more info on loading.

### Use ontology 🤸

In the previous section, we loaded an ontology from Obographs JSON file.
Now we have an instance of [`crate::ontology::csr::MinimalCsrOntology`] that can 
be used for various tasks.

#### Work with ontology terms

`MinimalCsrOntology` implements [`crate::ontology::OntologyTerms`] trait,
to support retrieval of specific terms by its index or `TermId`, and to iterate 
over all terms and `TermId`s.

We can get a term by its `TermId`:

```rust
# use std::fs::File;
# use std::io::BufReader;
# use flate2::bufread::GzDecoder;
# use ontolius::io::OntologyLoaderBuilder;
# use ontolius::ontology::csr::MinimalCsrOntology;
# let loader = OntologyLoaderBuilder::new().obographs_parser().build();
# let reader = GzDecoder::new(BufReader::new(File::open("resources/hp.small.json.gz").unwrap()));
# let hpo: MinimalCsrOntology = loader.load_from_read(reader)
#                                    .expect("HPO should be loaded");
#
use ontolius::TermId;
use ontolius::term::MinimalTerm;
use ontolius::ontology::OntologyTerms;

// `HP:0001250` corresponds to `Arachnodactyly``
let term_id: TermId = "HP:0001166".parse().unwrap();

// Get the term by its term ID ...
let term = hpo.term_by_id(&term_id);
assert!(term.is_some());

/// ... and check its name.
let term = term.unwrap();
assert_eq!(term.name(), "Arachnodactyly");
```

or iterate over the all ontology terms or their corresponding term IDs:

```rust
# use std::fs::File;
# use std::io::BufReader;
# use flate2::bufread::GzDecoder;
# use ontolius::io::OntologyLoaderBuilder;
# use ontolius::ontology::csr::MinimalCsrOntology;
# let loader = OntologyLoaderBuilder::new().obographs_parser().build();
# let reader = GzDecoder::new(BufReader::new(File::open("resources/hp.small.json.gz").unwrap()));
# let hpo: MinimalCsrOntology = loader.load_from_read(reader)
#                                    .expect("HPO should be loaded");
#
use ontolius::ontology::OntologyTerms;

// The toy HPO contains 614 terms and primary term ids,
let terms: Vec<_> = hpo.iter_terms().collect();
assert_eq!(terms.len(), 614);
assert_eq!(hpo.iter_term_ids().count(), 614);

// and the total of 1121 term ids (primary + obsolete)
assert_eq!(hpo.iter_all_term_ids().count(), 1121);
```

See [`crate::ontology::OntologyTerms`] trait for more details.

### Browse the hierarchy

`ontolius` enables to leverage the ontology hierarchy
via several traits. This typically includes iteration over term's parents, ancestors, children, or descendants.

The [`crate::ontology::HierarchyWalks`] trait supports iterating through [`TermId`]s whereas [`crate::ontology::HierarchyTraversals`] enables iteration over ontology graph indices. Iterating over indices is slightly faster and can be useful if we do not really care about the actual term IDs (e.g. to test if term `a` is an ancestor of term `b`).

The [`crate::ontology::HierarchyQueries`] simplifies testing if term `a` is a parent, child, ancestor, or descendant of term `b`.

In all cases, the hierarchy is represented as a directed acyclic graph that is built from `is_a` relationships.

Let's see how to use the ontology hierarchy. For instance, we can use [`crate::ontology::HierarchyWalks::iter_parent_ids`] to get parent ids of a term:

```rust
# use std::fs::File;
# use std::io::BufReader;
# use flate2::bufread::GzDecoder;
# use ontolius::io::OntologyLoaderBuilder;
# use ontolius::ontology::csr::MinimalCsrOntology;
# use ontolius::TermId;
# use ontolius::term::MinimalTerm;
# let loader = OntologyLoaderBuilder::new().obographs_parser().build();
# let reader = GzDecoder::new(BufReader::new(File::open("resources/hp.small.json.gz").unwrap()));
# let hpo: MinimalCsrOntology = loader.load_from_read(reader)
#                                    .expect("HPO should be loaded");

use ontolius::ontology::{HierarchyWalks, OntologyTerms};


let arachnodactyly: TermId = "HP:0001166".parse()
                                .expect("CURIE should be valid");

let parent_names: Vec<_> = hpo.iter_parent_ids(&arachnodactyly)
                               .map(|idx| hpo.term_by_id(idx).expect("A term for a term ID obtained from ontology should always be present"))
                               .map(MinimalTerm::name)
                               .collect();
assert_eq!(vec!["Slender finger", "Long fingers"], parent_names);
```

We first create the `TermId` that corresponds to *Arachnodactyly* and then we query `hpo` for its parents by calling `iter_parent_ids`. We retrieve the term that corresponds to term id, extract its name, and collect the names into a vector.

Similar methods exist for getting term IDs of ancestors, children, and descendants of a term. See [`crate::ontology::HierarchyWalks`] for more info.


## Supported ontologies

At this time, support for the following ontologies is tested:

* Human Phenotype Ontology (HPO)
* Gene Ontology (GO)
* Medical Action Ontology (MAxO)

Other ontologies are very likely to work too.
In case of any problems, please let us know on our [Issue tracker](https://github.com/ielis/ontolius/issues).


## Features

Ontolius includes several features, with the features marked by `(*)` being enabled
by default:

* `csr` `(*)` - include [`crate::ontology::csr`] module
  with implementation of ontology with graph backed by a CSR adjacency matrix
* `obographs` `(*)` - support loading Ontology from Obographs JSON file
* `pyo3` - include [`crate::py`] module with PyO3 bindings
  to selected data structs to support using from Python


## Run tests

The tests can be run by invoking:

```shell
cargo test
```

## Run benches

We use `criterion` for crate benchmarks.

Run the following to run the bench suite:

```shell
cargo bench
```

The benchmark report will be written into the `target/criterion/report` directory.
