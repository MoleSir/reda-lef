use crate::{LefVia, LefViaGenerateRule, LefViaRule, LefViaShape};
use super::LefTechnologyReader;
use crate::si2;
use super::utils;
use std::os::raw::c_int;

impl LefTechnologyReader {
    pub unsafe extern "C" fn read_via(_: si2::lefrCallbackType_e, obj: *mut si2::lefiVia, ud: *mut ::std::os::raw::c_void) -> c_int {
        unsafe {
            let reader = &mut *(ud as *mut Self);

            let via_name = utils::const_c_char_ptr_to_string(si2::lefiVia_name(obj));
            let mut via = LefVia::default();

            via.is_default = si2::lefiVia_hasDefault(obj) != 0;
            
            if si2::lefiVia_hasResistance(obj) != 0 {
                via.resistance = Some(si2::lefiVia_resistance(obj));
            }

            for l in 0..si2::lefiVia_numLayers(obj) {
                let layer_name = utils::const_c_char_ptr_to_string(si2::lefiVia_layerName(obj, l));
                let mut shapes = vec![];
                for r in 0..si2::lefiVia_numRects(obj, l) {
                    let xl = si2::lefiVia_xl(obj, l, r);
                    let yl = si2::lefiVia_yl(obj, l, r);
                    let xh = si2::lefiVia_xh(obj, l, r);
                    let yh = si2::lefiVia_yh(obj, l, r);
                    shapes.push(LefViaShape::Rect((xl, yl), (xh, yh)));
                }
                for p in 0..si2::lefiVia_numPolygons(obj, l) {
                    let poly = si2::lefiVia_getPolygon(obj, l, p);
                    let points = (0..poly.numPoints as usize)
                        .map(|i| (*poly.x.add(i), *poly.y.add(i)))
                        .collect();
                    shapes.push(LefViaShape::Polygon(points));
                }
                via.geometry.insert(layer_name, shapes);
            }

            reader.lef.vias.insert(via_name, via);
        }
        
        0
    }

    pub unsafe extern "C" fn read_viarule(_: si2::lefrCallbackType_e, obj: *mut si2::lefiViaRule, ud: *mut ::std::os::raw::c_void) -> c_int {
        unsafe {
            let reader = &mut *(ud as *mut Self);

            let via_name = utils::const_c_char_ptr_to_string(si2::lefiViaRule_name(obj));
            let mut via_rule = LefViaGenerateRule::default();

            assert!(si2::lefiViaRule_hasGenerate(obj) != 0);
            
            via_rule.is_default = si2::lefiViaRule_hasDefault(obj) != 0;
            via_rule.rule_name = via_name.clone();

            assert_eq!(si2::lefiViaRule_numLayers(obj), 3);
            let layer0 = si2::lefiViaRule_layer(obj, 0);
            let layer1 = si2::lefiViaRule_layer(obj, 1);
            let layer2 = si2::lefiViaRule_layer(obj,2);

            // LAYERS
            let layer0_name = utils::const_c_char_ptr_to_string(si2::lefiViaRuleLayer_name(layer0));
            let layer1_name = utils::const_c_char_ptr_to_string(si2::lefiViaRuleLayer_name(layer1));
            let layer2_name = utils::const_c_char_ptr_to_string(si2::lefiViaRuleLayer_name(layer2));
            via_rule.layers = (layer0_name, layer1_name, layer2_name);

            // ENCLOSURE
            if si2::lefiViaRuleLayer_hasEnclosure(layer0) != 0 {
                via_rule.enclosure.0 = (si2::lefiViaRuleLayer_enclosureOverhang1(layer0), si2::lefiViaRuleLayer_enclosureOverhang2(layer0));
            }
            if si2::lefiViaRuleLayer_hasEnclosure(layer1) != 0 {
                via_rule.enclosure.1 = (si2::lefiViaRuleLayer_enclosureOverhang1(layer1), si2::lefiViaRuleLayer_enclosureOverhang2(layer1));
            }

            // WIDTH
            if si2::lefiViaRuleLayer_hasWidth(layer0) != 0 {
                via_rule.width.0 = (si2::lefiViaRuleLayer_widthMin(layer0), si2::lefiViaRuleLayer_widthMax(layer0));
            }
            if si2::lefiViaRuleLayer_hasWidth(layer1) != 0 {
                via_rule.width.1 = (si2::lefiViaRuleLayer_widthMin(layer1), si2::lefiViaRuleLayer_widthMax(layer1));
            }

            // RECT
            if si2::lefiViaRuleLayer_hasRect(layer2) != 0 {
                via_rule.rect.0 = (si2::lefiViaRuleLayer_xl(layer2), si2::lefiViaRuleLayer_yl(layer2));
                via_rule.rect.1 = (si2::lefiViaRuleLayer_xh(layer2), si2::lefiViaRuleLayer_yh(layer2));
            }
            
            // SPACING
            if si2::lefiViaRuleLayer_hasSpacing(layer2) != 0 {
                via_rule.spacing.0 = si2::lefiViaRuleLayer_spacingStepX(layer2);
                via_rule.spacing.1 = si2::lefiViaRuleLayer_spacingStepY(layer2);
            }
            
            reader.lef.via_rules.insert(via_name, LefViaRule::Generate(via_rule));
        }
        
        0
    }
}