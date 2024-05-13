# Small HPO annotations file

`phenotype.real-shortlist.hpoa` contains 2 diseases from the HPO annotation file.

# Small HPO ontology

The ontology that contains the terms used in `phenotype.real-shortlist.hpoa` annotation file.

```shell
robot extract --input-iri https://github.com/obophenotype/human-phenotype-ontology/releases/download/v2023-04-05/hp-base.owl \
  -T hp.small.term_ids.txt -o hp.small.owl --method BOT --copy-ontology-annotations true
obographs convert -f json hp.small.owl
rm hp.small.owl
```
