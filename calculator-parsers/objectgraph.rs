
use tables::objects::ObjectGraph;

use super::*;



pub fn objectgraph() -> ObjectGraph {
    const EN: Language = const{Language::new("en").unwrap()};
    const LR: LanguageResolution = const{LanguageResolution{primary: Language::new("en").unwrap(), secondary: None}};

    let mut graph = ObjectGraph::new();

    fn new(graph: &mut ObjectGraph, name: &str, def: &str) {
        graph.define_new_object(
            EN,
            Name::new(name).unwrap_or_else(|| panic!("Cannot create name {name}")),
            def.into(),
        )
            .unwrap_or_else(|_| panic!("Cannot add new {name}"));
    }
    fn child(graph: &mut ObjectGraph, child: &str, parent: &str) {
        graph.claim_child(
            graph.find_name(child, &LR).unwrap_or_else(|| panic!("Cannot find child {child}")),
            graph.find_name(parent, &LR).unwrap_or_else(|| panic!("Cannot find parent {parent}"))
        ).unwrap_or_else(|_| panic!("Cannot claim child {} of {}", child, parent));
    }

    new(&mut graph, "Chemical matter", "Tangible persistent matter with intrinsic qualities (like taste, density, and reactivity) that can be separated, purified, and transformed into new kinds of chemical matter with entirely different qualities");

    new(&mut graph, "Substance", "Chemical matter with uniform composition throughout");
    new(&mut graph, "Mixture", "Chemical matter formed by combining two or more substances in variable proportions, exhibiting properties that vary continuously with composition");

    new(&mut graph, "Water", "The naturally occurring material found in rain, rivers, lakes, seas, and in the form of ice and vapor");
    new(&mut graph, "Distilled water", "Purified water free of dissolved minerals and impurities, acting as a strongly polar protic solvent");
    new(&mut graph, "Ethanol", "A volatile, flammable organic alcohol with both a polar hydroxyl group and a nonpolar hydrocarbon chain, acting as a moderately polar solvent");
    new(&mut graph, "Isopropanol", "A colorless, flammable secondary alcohol with moderate polarity, capable of dissolving a wide range of nonpolar and moderately polar compounds");
    new(&mut graph, "Hexane", "A volatile, colorless liquid alkane composed purely of carbon and hydrogen atoms, acting as a strongly nonpolar hydrocarbon solvent");
    new(&mut graph, "Cyclohexane", "A colorless, flammable cyclic hydrocarbon liquid with a ring structure, used as a standard nonpolar organic solvent");

    new(&mut graph, "Sodium chloride", "An ionic crystalline solid composed of positively charged sodium ions and negatively charged chloride ions arranged in a cubic lattice");
    new(&mut graph, "Benzoic acid", "A white crystalline organic solid consisting of a nonpolar aromatic benzene ring attached to a polar carboxyl group");
    new(&mut graph, "Salicylic acid", "A lipophilic crystalline organic acid featuring a benzene ring bonded to both a hydroxyl group and a carboxyl group");
    new(&mut graph, "Naphthalene", "A white, volatile crystalline aromatic hydrocarbon composed of two fused benzene rings, exhibiting purely nonpolar molecular interactions");
    new(&mut graph, "Paraffin wax", "A soft, colorless solid mixture of heavy, straight-chain saturated hydrocarbons that is completely nonpolar and water-insoluble");

    child(&mut graph, "Substance", "Chemical matter");
    child(&mut graph, "Mixture", "Chemical matter");

    child(&mut graph, "Distilled water", "Water");
    child(&mut graph, "Water", "Substance");
    child(&mut graph, "Ethanol", "Substance");
    child(&mut graph, "Isopropanol", "Substance");
    child(&mut graph, "Hexane", "Substance");
    child(&mut graph, "Cyclohexane", "Substance");
    child(&mut graph, "Sodium chloride", "Substance");
    child(&mut graph, "Benzoic acid", "Substance");
    child(&mut graph, "Salicylic acid", "Substance");
    child(&mut graph, "Naphthalene", "Substance");

    child(&mut graph, "Paraffin wax", "Chemical matter");

    graph
}
