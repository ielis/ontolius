# ontolius

`ontolius` is a crate to empower the algorithms that use Human Phenotype Ontology (HPO).

## WARNING

The project is under development, with features generally incomplete and subject to **CHANGE AT ANY TIME**.
The crate is *NOT* deployed on `crates.io` yet.

## Usage

We recommend adding the following into your `Cargo.toml` file:

```toml
ontolius = { git = 'https://github.com/ielis/ontolius.git', tag = 'v0.1.2' }
```

The `obographs` feature is enabled by deafult, to allow reading HPO from Obographs JSON file.

## Examples

We provide examples of *loading* ontology and its suggested usage
in applications.

### Load HPO

`ontolius` can load HPO from Obographs JSON file 
and the input file will be decompressed on the fly,
as long as the file ends with a `*.gz` suffix.

We can load the JSON file as follows:

```rust
use curie_util::TrieCurieUtil;
use ontolius::io::obographs::ObographsParser;
use ontolius::io::OntologyLoaderBuilder;
use ontolius::ontology::csr::CsrOntology;

// Load a toy Obographs file from the repo
let path = "resources/hp.small.json.gz";

// Configure the loader to parse the input as an Obographs file
let loader = OntologyLoaderBuilder::new()
                .parser(ObographsParser::new(TrieCurieUtil::default()))
                .build();

let hpo: CsrOntology<usize, _> = loader.load_from_path(path)
                                    .expect("HPO should be loaded");
```

We loaded an ontology from a toy JSON file. 
During the load, each term is assigned a numeric index and the indices are used as vertices 
of the ontology graph. 

As the name suggests, the hierarchy graph of the `CsrOntology` 
is backed by an adjacency matrix in compressed sparse row (CSR) format.
However, the backing data structure should be treated as an implementation detail.
Note that `CsrOntology` implements the [`crate::ontology::Ontology`] trait
which is the API the client code should use. 

Now let's move on to the example usage.

### Use HPO

In the previous section, we loaded an ontology from Obographs JSON file.
Now we have an instance of [`crate::ontology::Ontology`] that can 
be used for various tasks.

Note, we will import the *prelude* [`crate::prelude`] to reduce the import boilerplate.

#### Work with ontology terms

[`crate::ontology::Ontology`] acts as a container of terms to support 
retrieval of specific terms by its index or `TermId`, and to iterate 
over all terms and `TermId`s. 

We can get a term by its `TermId`: 

```rust
# use curie_util::TrieCurieUtil;
# use ontolius::io::obographs::ObographsParser;
# use ontolius::io::OntologyLoaderBuilder;
# use ontolius::ontology::csr::CsrOntology;
# let loader = OntologyLoaderBuilder::new().parser(ObographsParser::new(TrieCurieUtil::default())).build();
# let hpo: CsrOntology<usize, _> = loader.load_from_path("resources/hp.small.json.gz")
#                                    .expect("HPO should be loaded");
#
use ontolius::prelude::*;

// `HP:0001250` corresponds to `Arachnodactyly``
let term_id: TermId = ("HP", "0001166").into();

// Get the term by its term ID and check the name. 
let term = hpo.id_to_term(&term_id).expect("Arachnodactyly should be present");

assert_eq!(term.name(), "Arachnodactyly");
```

or iterate over the all ontology terms or their corresponding term IDs:

```rust
# use curie_util::TrieCurieUtil;
# use ontolius::io::obographs::ObographsParser;
# use ontolius::io::OntologyLoaderBuilder;
# use ontolius::ontology::csr::CsrOntology;
# let loader = OntologyLoaderBuilder::new().parser(ObographsParser::new(TrieCurieUtil::default())).build();
# let hpo: CsrOntology<usize, _> = loader.load_from_path("resources/hp.small.json.gz")
#                                    .expect("HPO should be loaded");
#
use ontolius::prelude::*;

// The toy HPO contains 614 terms and primary term ids,
let terms: Vec<_> = hpo.iter_terms().collect();
assert_eq!(terms.len(), 614);
assert_eq!(hpo.iter_term_ids().count(), 614);

// and the total of 1121 term ids (primary + obsolete)
assert_eq!(hpo.iter_all_term_ids().count(), 1121);
```

See [`crate::ontology::HierarchyAware`] for more details.

### Browse the hierarchy

`ontolius` models the ontology hierarchy using the [`crate::hierarchy::OntologyHierarchy`] trait, 
an instance of which is available from `Ontology`. 
The hierarchy is represented as a directed acyclic graph that is built from `is_a` relationships. 
The graph vertices correspond to term indices (not `TermId`s) that are determined 
when the ontology is built.

All methods of the ontology hierarchy operate in the term index space. The indices have 
all properties of `TermId`s, and can, therefore, be used *in lieu* of the `TermId`s. 

Let's see how to use the ontology hierarchy. For instance, to get the parent terms of a term.

```rust
# use curie_util::TrieCurieUtil;
# use ontolius::io::obographs::ObographsParser;
# use ontolius::io::OntologyLoaderBuilder;
# use ontolius::ontology::csr::CsrOntology;
# let loader = OntologyLoaderBuilder::new().parser(ObographsParser::new(TrieCurieUtil::default())).build();
# let hpo: CsrOntology<usize, _> = loader.load_from_path("resources/hp.small.json.gz")
#                                    .expect("HPO should be loaded");
#
use ontolius::prelude::*;

let hierarchy = hpo.hierarchy();

let arachnodactyly: TermId = ("HP", "0001166").into();

let idx = hpo.id_to_idx(&arachnodactyly)
            .expect("Arachnodacyly should be in HPO");
let parents: Vec<_> = hierarchy.parents_of(idx)
                        .flat_map(|idx| hpo.idx_to_term(*idx))
                        .collect();
let names: Vec<_> = parents.iter().map(|term| term.name()).collect();
assert_eq!(vec!["Slender finger", "Long fingers"], names);
```

Similar methods exist for getting ancestors, children, and descendent terms.
See [`crate::hierarchy::OntologyHierarchy`] for more details.

That's it for now.

## Features

Ontolius includes several features, with the features marked by `(*)` being enabled
by default:

* `obographs` `(*)` - support loading Ontology from Obographs JSON file
* `pyo3` - add PyO3 bindings to selected data structs to support using from Python


## Run tests

The tests can be run by invoking:

```shell
cargo test
```

## Run benches

We use `criterion` for crate benchmarks.

Run the following to run the bench suite:

```shell
cargo bench --features obographs
```

The benchmark report will be written into the `target/criterion/report` directory.
