mod csr {

    use ontolius::ontology::{
        csr::MinimalCsrOntology, OntologyTerms, TaxonomyQuery, TaxonomyTraversal, TaxonomyWalk,
    };
    #[allow(deprecated)]
    use ontolius::ontology::{HierarchyQueries, HierarchyWalks};

    #[test]
    fn test_csr_ontology_can_be_used_with_trait_bounds() {
        let o: Option<MinimalCsrOntology> = None;

        if o.is_some() {
            // This will never run. It does not matter,
            // since it is enough that the code compiles.
            let o: &MinimalCsrOntology = o.as_ref().unwrap();

            pretend_to_use_taxonomy_query(o);
            pretend_to_use_taxonomy_walk(o);
            pretend_to_use_taxonomy_traversal(o);
            pretend_to_use_ontology_terms(o);
            pretend_to_use_hierarchy_queries(o);
            pretend_to_use_hierarchy_walks(o);
        }

        fn pretend_to_use_taxonomy_query(_o: impl TaxonomyQuery) {}
        fn pretend_to_use_taxonomy_walk(_o: impl TaxonomyWalk) {}
        fn pretend_to_use_taxonomy_traversal(_o: impl TaxonomyTraversal) {}
        fn pretend_to_use_ontology_terms(_o: impl OntologyTerms) {}
        #[allow(deprecated)]
        fn pretend_to_use_hierarchy_queries(_o: impl HierarchyQueries) {}
        #[allow(deprecated)]
        fn pretend_to_use_hierarchy_walks(_o: impl HierarchyWalks) {}
    }
}
