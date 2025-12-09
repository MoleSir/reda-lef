use crate::{LefCutLayer, LefCutSpacingRule, LefLayer, LefRoutingDirection, LefRoutingLayer, LefSpacingRules, LefSpacingTable, LefSpacingType};
use super::{LefReadResult, LefTechnologyReader};
use crate::si2;
use super::utils;
use std::{os::raw::{c_int, c_void}, str::FromStr};
use paste::paste;

macro_rules! layer_attr_opt {
    ($layer:ident, $si2_obj:ident, $field_name:ident, $attr_name:ident, $as_ty:ty) => {
        paste! {
            if si2::[< lefiLayer_has $attr_name:camel >]($si2_obj) != 0 { 
                $layer.$field_name = Some(si2::[< lefiLayer_ $attr_name >]($si2_obj) as $as_ty);
            }
        }
    };
    
    ($layer:ident, $si2_obj:ident, $field_name:ident, $attr_name:ident) => {
        paste! {
            if si2::[< lefiLayer_has $attr_name:camel >]($si2_obj) != 0 {
                $layer.$field_name = Some(si2::[< lefiLayer_ $attr_name >]($si2_obj));
            }
        }
    };
}

macro_rules! layer_attr {
    ($layer:ident, $si2_obj:ident, $field_name:ident, $attr_name:ident) => {
        paste! {
            if si2::[< lefiLayer_has $attr_name:camel >]($si2_obj) != 0 { 
                $layer.$field_name = si2::[< lefiLayer_ $attr_name >]($si2_obj);
            }
        }
    };
}

impl LefTechnologyReader {
    pub unsafe extern "C" fn read_layer(_: si2::lefrCallbackType_e, obj: *mut si2::lefiLayer, ud: *mut c_void) -> c_int {
        unsafe {
            let reader = &mut *(ud as *mut Self);
            si2::lefiLayer_width(obj);
            let tpe =  utils::const_c_char_ptr_to_str(si2::lefiLayer_type(obj));
            match tpe {
                "CUT" => {
                    match Self::read_cut_layer(obj) {
                        Ok(layer) => reader.lef.layers.push(LefLayer::Cut(layer)),
                        Err(err) => { 
                            reader.error = Some(err);
                            return 1;
                        }
                    }
                }
                "ROUTING" => {
                    match Self::read_routing_layer(obj) {
                        Ok(layer) => reader.lef.layers.push(LefLayer::Routing(layer)),
                        Err(err) => { 
                            reader.error = Some(err);
                            return 1;
                        }
                    }
                }
                "MASTERSLICE" => {

                }
                "OVERLAP" => {

                }
                _ => panic!(),
            }
        }
        0
    }

    unsafe fn read_routing_layer(obj: *mut si2::lefiLayer) -> LefReadResult<LefRoutingLayer> {
        unsafe {
            let mut layer = LefRoutingLayer::default();
            layer.name = utils::const_c_char_ptr_to_string(si2::lefiLayer_name(obj));

            layer_attr_opt!(layer, obj, mask_num, mask, u32);
            layer_attr!(layer, obj, width, width);           
            layer_attr_opt!(layer, obj, min_area, area);           
            layer_attr_opt!(layer, obj, max_width, maxwidth);      
            layer_attr_opt!(layer, obj, min_width, minwidth);      
            layer_attr_opt!(layer, obj, height, height);           
            layer_attr_opt!(layer, obj, resistance, resistancePerCut);
            layer_attr_opt!(layer, obj, resistance, resistance);
            layer_attr_opt!(layer, obj, capacitance, capacitance); 
            layer_attr_opt!(layer, obj, thickness, thickness);
            layer_attr_opt!(layer, obj, edge_capacitance, edgeCap);

            if si2::lefiLayer_hasDirection(obj) != 0 { // DIRECTION
                let direction = utils::const_c_char_ptr_to_str(si2::lefiLayer_direction(obj));
                layer.direction = LefRoutingDirection::from_str(&direction).unwrap();   
            }
            if si2::lefiLayer_hasPitch(obj) != 0 { // PITCH
                let pitch = si2::lefiLayer_pitch(obj);
                layer.pitch = (pitch, pitch);
            } else if si2::lefiLayer_hasXYPitch(obj) != 0 {
                let x_pitch = si2::lefiLayer_pitchX(obj);
                let y_pitch = si2::lefiLayer_pitchY(obj);
                layer.pitch = (x_pitch, y_pitch);
            } 
            if si2::lefiLayer_hasOffset(obj) != 0 { // OFFSET
                let offset = si2::lefiLayer_offset(obj);
                layer.offset = Some((offset, offset));
            } else if si2::lefiLayer_hasXYOffset(obj) != 0 {
                let x_offset = si2::lefiLayer_offsetX(obj);
                let y_offset = si2::lefiLayer_offsetY(obj);
                layer.offset = Some((x_offset, y_offset));
            } 
            if si2::lefiLayer_hasSpacingNumber(obj) != 0 { // SPACING
                for index in 0..si2::lefiLayer_numSpacing(obj) {
                    let min_spacing = si2::lefiLayer_spacing(obj, index);
                    let spacing_type = if si2::lefiLayer_hasSpacingRange(obj, index) != 0 {
                        let min_width = si2::lefiLayer_spacingRangeMin(obj, index);
                        let max_width = si2::lefiLayer_spacingRangeMax(obj, index);
                        // TODO: range type
                        Some(LefSpacingType::Range { min_width, max_width, spacing_range_type: None })
                    } else if si2::lefiLayer_hasSpacingEndOfLine(obj, index) != 0{
                        let eol_width = si2::lefiLayer_spacingEolWidth(obj, index);
                        let eol_widthing = si2::lefiLayer_spacingEolWithin(obj, index);
                        Some(LefSpacingType::EndOfLine { eol_width, eol_widthing })
                    } else if si2::lefiLayer_hasSpacingSamenet(obj, index) != 0{
                        let power_ground_only = si2::lefiLayer_hasSpacingSamenetPGonly(obj, index) != 0;
                        Some(LefSpacingType::SameNet { power_ground_only })
                    } else if si2::lefiLayer_hasSpacingNotchLength(obj, index) != 0{
                        let min_notch_length = si2::lefiLayer_spacingNotchLength(obj, index);
                        Some(LefSpacingType::NotchLength { min_notch_length })
                    } else if si2::lefiLayer_hasSpacingEndOfNotchWidth(obj, index) != 0{
                        let end_of_notch_width = si2::lefiLayer_spacingEndOfNotchWidth(obj, index);
                        let min_notch_spacing = si2::lefiLayer_spacingEndOfNotchSpacing(obj, index);
                        let min_notch_length = si2::lefiLayer_spacingEndOfNotchLength(obj, index);
                        Some(LefSpacingType::EndOfNotchWidth { end_of_notch_width, min_notch_spacing, min_notch_length })
                    } else {    
                        None
                    };
                    layer.spacing.push(LefSpacingRules { min_spacing, spacing_type });
                }
            }
            if si2::lefiLayer_numSpacingTable(obj) == 1 { // SPACINGTABLE
                let table = si2::lefiLayer_spacingTable(obj, 0);
                if si2::lefiSpacingTable_isInfluence(table) != 0 {
                    todo!();
                } else if si2::lefiSpacingTable_isParallel(table) != 0 {
                    let parallel = si2::lefiSpacingTable_parallel(table);
                    
                    let parallel_run_lengths = (0..si2::lefiParallel_numLength(parallel))
                        .map(|col| si2::lefiParallel_length(parallel, col))
                        .collect();

                    let widths = (0..si2::lefiParallel_numWidth(parallel))
                        .map(|row| si2::lefiParallel_width(parallel, row))
                        .collect();
                    
                    let spacings: Vec<Vec<_>> = (0..si2::lefiParallel_numWidth(parallel))
                        .map(|row| {
                            (0..si2::lefiParallel_numLength(parallel))
                                .map(|col| si2::lefiParallel_widthSpacing(parallel, row, col))
                                .collect::<Vec<f64>>()
                        })
                        .collect();
                    
                    layer.spacing_table = Some(LefSpacingTable { parallel_run_lengths, widths, spacings });
                }                
            }
    
            Ok(layer)
        }
    }

    unsafe fn read_cut_layer(obj: *mut si2::lefiLayer) -> LefReadResult<LefCutLayer> {
        unsafe {
            let mut layer = LefCutLayer::default();
            layer.name = utils::const_c_char_ptr_to_string(si2::lefiLayer_name(obj));
            
            layer_attr_opt!(layer, obj, mask_num, mask, u32);
            layer_attr_opt!(layer, obj, width, width);
            layer_attr_opt!(layer, obj, resistance, resistance);

            if si2::lefiLayer_hasSpacingNumber(obj) != 0 {
                for index in 0..si2::lefiLayer_numSpacing(obj) {
                    let spacing = si2::lefiLayer_spacing(obj, index);
                    let center_to_center = si2::lefiLayer_hasSpacingCenterToCenter(obj, index) != 0;
                    let same_net = si2::lefiLayer_hasSpacingSamenet(obj, index) != 0;
                    layer.spacing.push(LefCutSpacingRule {
                        spacing, center_to_center, same_net
                    });
                }
            }

            Ok(layer)
        }
    }
}