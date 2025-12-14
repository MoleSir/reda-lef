use crate::{read::LefReadError, LefGeometry, LefLayerGeometries, LefMacro, LefMacroPin, LefOrient, LefPinDirection, LefPinShape, LefSignalUse, LefSite, LefStepPattern, LefSymmetry};
use super::LefCellLibraryReader;
use crate::si2;
use super::utils;
use std::{os::raw::{c_int, c_void}, str::FromStr};

impl LefCellLibraryReader {
    pub unsafe extern "C" fn read_macro(_: si2::lefrCallbackType_e, obj: *mut si2::lefiSite, ud: *mut c_void) -> c_int {
        unsafe {
            let reader = &mut *(ud as *mut Self);

            let mut makcro = LefMacro::default();
            makcro.name = utils::const_c_char_ptr_to_string(si2::lefiMacro_name(obj));

            // CLASS
            if si2::lefiMacro_hasClass(obj) != 0 {
                match FromStr::from_str(utils::const_c_char_ptr_to_str(si2::lefiMacro_macroClass(obj))) {
                    Ok(class) => makcro.class = Some(class),
                    Err(err) => {
                        reader.error = Some(LefReadError::Msg(err));
                        return 1;
                    }
                }
            } 

            // FOREIGN
            if si2::lefiMacro_hasForeign(obj) != 0 {
                for index in 0..si2::lefiMacro_numForeigns(obj) {
                    let name = utils::const_c_char_ptr_to_string(si2::lefiMacro_foreignName(obj, index));
                    let (pos, orient) = if si2::lefiMacro_hasForeignPoint(obj, index) != 0 {
                        let pos = (si2::lefiMacro_foreignX(obj, index), si2::lefiMacro_foreignY(obj, index));
                        if si2::lefiMacro_hasForeignOrient(obj, index) != 0 {
                            (pos, LefOrient::from_str(utils::const_c_char_ptr_to_str(si2::lefiMacro_foreignOrientStr(obj, index))).unwrap())
                        } else {
                            (pos, LefOrient::default())
                        }
                    } else {
                        ((0.0, 0.0), LefOrient::default()) 
                    };
                    
                    makcro.foreign.push((name, pos, orient));
                }
            }

            // ORIGIN
            if si2::lefiMacro_hasOrigin(obj) != 0 {
                makcro.origin = (si2::lefiMacro_originX(obj), si2::lefiMacro_originY(obj))
            }

            // EEQ
            if si2::lefiMacro_hasEEQ(obj) != 0 {
                makcro.eeq = Some(utils::const_c_char_ptr_to_string(si2::lefiMacro_EEQ(obj)));
            }
            
            // SIZE
            if si2::lefiMacro_hasSize(obj) != 0 {
                makcro.size = Some((si2::lefiMacro_sizeX(obj), si2::lefiMacro_sizeY(obj)));
            }

            // SYMMETRY
            makcro.symmetry = LefSymmetry {
                x: si2::lefiMacro_hasXSymmetry(obj) != 0,
                y: si2::lefiMacro_hasYSymmetry(obj) != 0,
                r90: si2::lefiMacro_has90Symmetry(obj) != 0,
            };

            // SITE
            if si2::lefiMacro_hasSiteName(obj) != 0 {
                let mut site = LefSite::default();
                site.name = utils::const_c_char_ptr_to_string(si2::lefiMacro_siteName(obj));
                makcro.sites.push(site);
            }
            for index in 0..si2::lefiMacro_numSitePattern(obj) {
                let mut pattern = LefSite::default();
                let pattern_obj = si2::lefiMacro_sitePattern(obj, index);
                pattern.name = utils::const_c_char_ptr_to_string(si2::lefiSitePattern_name(pattern_obj));
                pattern.origin = (si2::lefiSitePattern_x(pattern_obj), si2::lefiSitePattern_y(pattern_obj));
                pattern.site_orient = LefOrient::from_str(utils::const_c_char_ptr_to_str(si2::lefiSitePattern_orientStr(pattern_obj))).unwrap();

                if si2::lefiSitePattern_hasStepPattern(pattern_obj) != 0 {
                    pattern.step_pattern = Some(LefStepPattern {
                        x_start: si2::lefiSitePattern_xStart(pattern_obj),
                        y_start: si2::lefiSitePattern_yStart(pattern_obj),
                        x_step: si2::lefiSitePattern_xStep(pattern_obj),
                        y_step: si2::lefiSitePattern_yStep(pattern_obj),
                    })
                }

                makcro.sites.push(pattern);
            }
   
            // PIN
            makcro.pins = reader.take_pins();
            
            // OBS
            makcro.obs = reader.take_geometries();

            // DENSITY

            // PROPERTY

            reader.lef.macros.insert(makcro.name.clone(), makcro);   
        }
        0
    }

    pub unsafe extern "C" fn read_pin(_: si2::lefrCallbackType_e, obj: *mut si2::lefiPin, ud: *mut c_void) -> c_int {
        unsafe {
            let reader = &mut *(ud as *mut Self);
            
            let mut pin = LefMacroPin::default();
            pin.name = utils::const_c_char_ptr_to_string(si2::lefiPin_name(obj));  

            if si2::lefiPin_hasDirection(obj) != 0 {
                let dir = LefPinDirection::from_str(&utils::const_c_char_ptr_to_str(si2::lefiPin_direction(obj))).unwrap();
                pin.direction = Some(dir);
            }

            if si2::lefiPin_hasUse(obj) != 0 {
                let uuse =  utils::const_c_char_ptr_to_str(si2::lefiPin_use(obj));
                let uuse = LefSignalUse::from_str(&uuse).unwrap();
                pin.signal_use = Some(uuse);
            }

            if si2::lefiPin_hasShape(obj) != 0 {
                let shape =  utils::const_c_char_ptr_to_str(si2::lefiPin_shape(obj));
                let shape = LefPinShape::from_str(&shape).unwrap();
                pin.shape_type = Some(shape);
            }

            if si2::lefiPin_hasMustjoin(obj) != 0 {
                pin.must_join = Some(utils::const_c_char_ptr_to_string(si2::lefiPin_mustjoin(obj)));
            }

            for index in 0..si2::lefiPin_numPorts(obj) {
                let port = si2::lefiPin_port(obj, index);
                pin.port = Self::read_geometries(port);
            }

            reader.pins.push(pin);
        }
        0
    }

    pub unsafe extern "C" fn read_obs(_: si2::lefrCallbackType_e, obj: *mut si2::lefiObstruction, ud: *mut c_void) -> c_int {
        unsafe {
            let reader = &mut *(ud as *mut Self);
            let obj = si2::lefiObstruction_geometries(obj);
            let geometries = Self::read_geometries(obj);   
            reader.geometries = geometries;                 
        }
        0
    }

    unsafe fn read_geometries(obj: *const si2::lefiGeometries) -> Vec<LefLayerGeometries> {
        unsafe {
            let mut all_geometries: Vec<LefLayerGeometries> = vec![];
            for index in 0..si2::lefiGeometries_numItems(obj) {
                let mut geometries = LefLayerGeometries::default();
                geometries.except_pg_net = si2::lefiGeometries_hasLayerExceptPgNet(obj, index) != 0;
                // let layer_min_spacing = si2::lefiGeometries_getLayerMinSpacing(obj, index);
                // let layer_rule_width = si2::lefiGeometries_getLayerRuleWidth(obj, index);
                
                match si2::lefiGeometries_itemType(obj, index) {
                    si2::lefiGeomEnum_lefiGeomClassE => {
                        println!("lefiGeomEnum_lefiGeomClassE");
                    }
                    si2::lefiGeomEnum_lefiGeomLayerE => {
                        geometries.layer_name = utils::const_c_char_ptr_to_string(si2::lefiGeometries_getLayer(obj, index));
                    }
                    si2::lefiGeomEnum_lefiGeomWidthE => {
                        geometries.width = Some(si2::lefiGeometries_getWidth(obj, index));
                    }
                    si2::lefiGeomEnum_lefiGeomPathE => {
                        unimplemented!("lefiGeomEnum_lefiGeomPathE");
                    }
                    si2::lefiGeomEnum_lefiGeomPathIterE => {
                        unimplemented!("lefiGeomEnum_lefiGeomPathIterE")
                    }
                    si2::lefiGeomEnum_lefiGeomRectE => {
                        let rect = si2::lefiGeometries_getRect(obj, index);
                        let rect = &*rect;
                        geometries.geometries.push(LefGeometry::Rect((rect.xl, rect.yl), (rect.xh, rect.yh)));
                    }
                    si2::lefiGeomEnum_lefiGeomRectIterE => {
                        unimplemented!("lefiGeomEnum_lefiGeomRectIterE")
                    }
                    si2::lefiGeomEnum_lefiGeomPolygonE => {
                        let polygon = si2::lefiGeometries_getPolygon(obj, index);
                        let polygon = &*polygon;
                        let mut points = vec![];
                        for j in 0..polygon.numPoints as usize {
                            points.push((*polygon.x.add(j), *polygon.y.add(j)));
                        }
                        geometries.geometries.push(LefGeometry::Polygon(points));
                    }
                    si2::lefiGeomEnum_lefiGeomPolygonIterE => {
                        unimplemented!("lefiGeomEnum_lefiGeomPolygonIterE")
                    }
                    si2::lefiGeomEnum_lefiGeomViaE => {
                        unimplemented!("lefiGeomEnum_lefiGeomViaE")
                    }
                    si2::lefiGeomEnum_lefiGeomViaIterE => {
                        unimplemented!("lefiGeomEnum_lefiGeomViaIterE")
                    }
                    _ => panic!(),
                };

                all_geometries.push(geometries);
            }
            
            all_geometries
        }
    }
}