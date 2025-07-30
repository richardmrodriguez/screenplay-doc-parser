use std::{
    collections::{HashMap, HashSet},
};

use crate::screenplay_document::{self, SPType};

// ------------ Get LOCATIONs...
// TODO: Filter Locations by Characters speaking in location...
// All "get location" or "filter location" funcs return a vec of &LocationID.

///
/// Returns an Optional Vec of &LocationID, which make up the path to this leaf node, in order from root to leaf.
///
pub fn get_full_location_path_for_leaf_node<'a>(
    screenplay_doc: &'a crate::screenplay_document::ScreenplayDocument,
    location: &'a screenplay_document::LocationID,
) -> Option<(Vec<&'a screenplay_document::LocationID>)> {
    let mut current_location_node_id: &screenplay_document::LocationID = location;
    let mut id_path: Vec<&screenplay_document::LocationID> = vec![current_location_node_id];
    loop {
        let Some(location_node) = screenplay_doc.locations.get(current_location_node_id) else {
            return None; // ERROR: BAD LOCATION ID!
        };
        let Some(superlocation) = &location_node.superlocation else {
            break; // done pushing to path vec
        };
        id_path.push(superlocation);
        current_location_node_id = superlocation;
    }

    if id_path.is_empty() {
        return None;
    };

    id_path.reverse();
    Some(id_path)
}

/// Returns the full Location path as a string (i.e. the full scene heading)
/// for a given Location leaf node.
pub fn get_full_string_for_location_path<'a>(
    screenplay_doc: &'a crate::screenplay_document::ScreenplayDocument,
    location_leaf_node: &'a screenplay_document::LocationID,
) -> Option<String> {
    let location_id_path =
        get_full_location_path_for_leaf_node(screenplay_doc, location_leaf_node)?;

    let mut path_string = String::new();
    for id in location_id_path {
        let Some(location_node) = screenplay_doc.locations.get(id) else {
            return None; // ERROR: BAD LOCATION ID!
        };
        if !path_string.is_empty() {
            path_string.push_str(" - ");
        }
        path_string.push_str(&location_node.string);
    }
    if path_string.is_empty() {
        return None; // ERROR: ???
    }

    Some(path_string)
}

/// Determines if a "location path" exists.
///
/// Returns `None` if nothing matches the root of the path.
///
/// Returns `Some((&LocationID, Vec<String>))` if a partial match is found.
/// The `&LocationID` is the last valid location the path matched.
/// The `Vec<String>` is the remaining subpath, which does not yet exist as `LocationNode`s.
///
/// If the full path exists, the `Vec<String>` will be empty.
///
/// The caller can afterwards handle creating the rest of the Location path,
/// and appending a new LocationID to the sublocations field
///
///
pub fn location_path_exists<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    path: &[String],
) -> Option<(&'a screenplay_document::LocationID, Vec<String>)> {
    if path.is_empty() {
        return None;
    }

    let path_root = &path[0];

    for (id, location) in &screenplay_document.locations {
        if location.string == *path_root {
            if path.len() == 1 {
                return Some((id, Vec::new()));
            }
            if path.len() > 1 && location.sublocations.is_empty() {
                return Some((id, Vec::from(&path[1..])));
            }

            return location.subpath_exists(&id, &path[1..], screenplay_document);
        }
    }

    None
}

pub fn get_locations_with_matching_str<'a>(
    screenplay_document: &'a screenplay_document::ScreenplayDocument,
    location_string: &'a String,
) -> Option<Vec<&'a screenplay_document::LocationID>> {
    let mut loc_id_vec: Vec<&screenplay_document::LocationID> = Vec::new();
    for (id, location) in &screenplay_document.locations {
        if location.string == *location_string {
            loc_id_vec.push(id);
        }
    }
    if loc_id_vec.is_empty() {
        return None;
    }
    Some(loc_id_vec)
}

/// Gets the Location ID of the root in this location path, from a given LocationID.
///
/// Returns `'None` if the LocationID is invalid.
pub fn get_location_root_for_node<'a>(
    screenplay_document: &'a screenplay_document::ScreenplayDocument,
    location_id: &'a screenplay_document::LocationID,
) -> Option<&'a screenplay_document::LocationID> {
    let location = screenplay_document.locations.get(location_id)?;
    let Some(superlocation) = &location.superlocation else {
        return Some(location_id);
    };
    return get_location_root_for_node(screenplay_document, &superlocation);
}

pub fn get_all_location_leafs<'a>(
    screenplay_document: &'a screenplay_document::ScreenplayDocument,
    location_id: &'a screenplay_document::LocationID,
) -> Option<Vec<&'a screenplay_document::LocationID>> {
    let Some(location) = screenplay_document.locations.get(location_id) else {
        return None;
    };
    if location.sublocations.is_empty() {
        return Some(vec![location_id]);
    }
    let mut location_id_set: HashSet<&screenplay_document::LocationID> = HashSet::new();
    for sublocation_id in &location.sublocations {
        let Some(subset) = get_all_location_leafs(screenplay_document, &sublocation_id) else {
            continue;
        };
        location_id_set.extend(subset);
    }
    if location_id_set.is_empty() {
        return None;
    }
    Some(location_id_set.iter().copied().collect())
}

pub fn filter_locations_by_character_speaking<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    locations_to_filter: Vec<&'a screenplay_document::LocationID>,
    character: &'a screenplay_document::Character,
) -> Option<Vec<&'a screenplay_document::LocationID>> {
    let scenes_with_character =
        get_all_scenes_with_character_speaking(screenplay_document, character)?;
    let mut filtered_locations: HashSet<&screenplay_document::LocationID> = HashSet::new();

    scenes_with_character
        .iter()
        .map(|(s, scn)| &scn.story_locations)
        .for_each(|lcv| {
            //TODO: filter this by ones that
            lcv.iter().for_each(|lc| {
                if locations_to_filter.contains(&lc) {
                    filtered_locations.insert(lc);
                }
            });
        });
    if filtered_locations.is_empty() {
        return None;
    }

    Some(filtered_locations.iter().copied().collect())
}

pub fn get_all_locations_with_character_speaking<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    character: &'a screenplay_document::Character,
) -> Option<Vec<&'a screenplay_document::LocationID>> {
    let all_locations = &screenplay_document.locations;
    let all_locations_vec: Vec<_> = all_locations.iter().map(|(id, scn)| id).collect();

    filter_locations_by_character_speaking(screenplay_document, all_locations_vec, character)
}

pub fn filter_locations_by_page_idx<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    locations: Vec<&'a screenplay_document::LocationID>,
    page: usize,
) -> Option<Vec<&'a screenplay_document::LocationID>> {
    let Some(locations_on_page) = get_all_locations_on_page_by_index(screenplay_document, page)
    else {
        return None;
    };

    let mut filtered_locations: Vec<&screenplay_document::LocationID> = Vec::new();

    for loc in locations {
        if locations_on_page.contains(&loc) {
            filtered_locations.push(loc);
        }
    }

    if filtered_locations.is_empty() {
        return None;
    }
    Some(filtered_locations)
}

pub fn get_all_locations_on_page_by_index(
    screenplay_document: &crate::screenplay_document::ScreenplayDocument,
    page: usize,
) -> Option<Vec<&screenplay_document::LocationID>> {
    let mut locations_on_page: HashSet<&screenplay_document::LocationID> = HashSet::new();
    let scenes = get_all_scenes_on_page_by_index(screenplay_document, page)?;
    scenes
        .iter()
        .filter_map(|(scene_id, scene)| {
            if scene.story_locations.is_empty() {
                return None;
            }

            Some(&scene.story_locations)
        })
        .for_each(|s| {
            s.iter().for_each(|l| {
                locations_on_page.insert(l);
            })
        });

    if locations_on_page.is_empty() {
        return None;
    }
    Some(locations_on_page.iter().copied().collect())
}

// ------------ Get LINEs...
// All "get_line..." or "filter_lines" funcs should return
// A) HashMap<ScreenplayCoordinate, &Line>
// B) Line

pub fn filter_lines_by_multiple_scenes<'a>(
    screenplay_document: &crate::screenplay_document::ScreenplayDocument,
    lines: &Vec<(screenplay_document::ScreenplayCoordinate, &'a screenplay_document::Line)>,
    scenes_filter: Vec<(&screenplay_document::SceneID, &screenplay_document::Scene)>,
) -> Option<Vec<(screenplay_document::ScreenplayCoordinate, &'a screenplay_document::Line)>> {
    if scenes_filter.is_empty() {
        //panic!("EMPTY FILTER!");
        return None;
    }

    let mut scn_filtered_lines: Vec<
        (screenplay_document::ScreenplayCoordinate,
        &screenplay_document::Line)
    > = Vec::new();

    lines
        .iter()
        .filter(|(coord, ln)| {
            let Some((scene_id, scene)) =
                get_scene_for_screenplay_coordinate(screenplay_document, &coord)
            else {
                return false;
            };
            for (fscn_id, _) in &scenes_filter {
                if *fscn_id == scene_id {
                    return true;
                }
            }
            false
        })
        .for_each(|(coord, line)| {
            scn_filtered_lines.push((coord.clone(), line));
        });

    if scn_filtered_lines.is_empty() {
        return None;
    }
    if scn_filtered_lines.len() == lines.len() {
        //panic!("DIDN'T FILTER ANY LINES AT ALL!!!");
    }

    //println!("\nDURATION TO FILTER BY SCENES: {:?}", bench_res);
    Some(scn_filtered_lines)
}

pub fn filter_lines_by_multiple_locations<'a>(
    screenplay_doc: &'a screenplay_document::ScreenplayDocument,
    lines: &Vec<(screenplay_document::ScreenplayCoordinate, &'a screenplay_document::Line)>,
    locations_filter: Vec<&screenplay_document::LocationID>,
) -> Option<Vec<(screenplay_document::ScreenplayCoordinate, &'a screenplay_document::Line)>> {
    let Some(scenes_ordered) = get_all_scenes_ordered(screenplay_doc) else {
        return None;
    };
    let Some(scenes_filtered) =
        filter_scenes_by_locations(screenplay_doc, scenes_ordered, locations_filter)
    else {
        return None;
    };
    return filter_lines_by_multiple_scenes(screenplay_doc, lines, scenes_filtered);
}



pub fn get_all_lines_of_dialogue_for_character<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    character: &'a screenplay_document::Character,
) -> Option<Vec<(screenplay_document::ScreenplayCoordinate, &'a screenplay_document::Line)>> {
    let mut lines_with_coords: Vec<(
        screenplay_document::ScreenplayCoordinate,
        &screenplay_document::Line)
    > = Vec::new();
    //let mut lines: Vec<&Line> = Vec::new();
    let mut is_dialogue = false;

    for (p_index, page) in screenplay_document.pages.iter().enumerate() {
        for (l_index, line) in page.lines.iter().enumerate() {
            if line.line_type != Some(SPType::SP_CHARACTER)
                && line.line_type != Some(SPType::SP_DUAL_CHARACTERS)
                && line.line_type != Some(SPType::SP_DIALOGUE)
                && line.line_type != Some(SPType::SP_DUAL_DIALOGUES)
            {
                continue;
            }
            //println!("MIGHT BE CHARACTER OR DIALOGUE");
            if character.is_line(line) {
                //println!("Oh boy!!!! | {:?} | {:?}", line.line_type, line.text_elements);
                is_dialogue = true;
                continue;
            }
            if !is_dialogue {
                continue;
            }
            match line.line_type {
                Some(SPType::SP_DIALOGUE) | Some(SPType::SP_DUAL_DIALOGUES) => {
                    lines_with_coords.push(
                        (screenplay_document::ScreenplayCoordinate {
                            page: p_index,
                            line: l_index,
                            element: None,
                        },
                        line)
                    );
                }
                _ => {
                    is_dialogue = false;
                    continue;
                }
            }
        }
    }
    if lines_with_coords.is_empty() {
        return None;
    }
    Some(lines_with_coords)
}

// ------------ Get CHARACTERS...
// All returns should be Vec<&Character>.
// TODO: Filter Characters by Scenes they speak in,
// Locations they speak in,

pub fn filter_characters_by_scene<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    characters: &Vec<&'a screenplay_document::Character>,
    scene_id: &screenplay_document::SceneID,
) -> Option<Vec<&'a screenplay_document::Character>> {
    let scn = screenplay_document.scenes.get(scene_id)?;
    let mut current_page = scn.start.page;
    let mut characters_in_scene: HashSet<&screenplay_document::Character> = HashSet::new();

    'seeking: loop {
        let Some(page) = screenplay_document.pages.get(current_page) else {
            break 'seeking;
        };
        'lines: for (l_idx, line) in page.lines.iter().enumerate() {
            if (current_page, l_idx) < (scn.start.page, scn.start.line) {
                continue 'lines;
            }
            if line.line_type
                == Some(SPType::SP_SCENE_HEADING(
                    screenplay_document::SceneHeadingElement::Line,
                ))
            {
                if (current_page, l_idx) > (scn.start.page, scn.start.line) {
                    break 'seeking;
                }
            }
            if line.line_type != Some(SPType::SP_CHARACTER) {
                continue 'lines;
            }
            if characters_in_scene.len() == characters.len() {
                break 'seeking;
            }
            for character in characters {
                if character.is_line(line) {
                    characters_in_scene.insert(&character);
                }
            }
        }
        current_page += 1;
    }
    if characters_in_scene.is_empty() {
        return None;
    }

    Some(characters_in_scene.iter().copied().collect())
}

pub fn get_characters_for_scene<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    scene_id: &screenplay_document::SceneID,
) -> Option<Vec<&'a screenplay_document::Character>> {
    let current_characters = &screenplay_document.characters.iter().collect();
    filter_characters_by_scene(screenplay_document, current_characters, scene_id)
}

pub fn filter_characters_by_page_index<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    characters_to_filter: Vec<&'a screenplay_document::Character>,
    page_index: usize,
) -> Option<Vec<&'a screenplay_document::Character>> {
    let mut characters_on_page: HashSet<&screenplay_document::Character> = HashSet::new();

    let Some(page) = screenplay_document.pages.get(page_index) else {
        return None;
    };
    'lines: for line in &page.lines {
        if line.line_type != Some(SPType::SP_CHARACTER) {
            continue 'lines;
        }
        if characters_on_page.len() == characters_to_filter.len() {
            break;
        }
        for character in &characters_to_filter {
            if character.is_line(line) {
                characters_on_page.insert(character);
            }
        }
    }

    if characters_on_page.is_empty() {
        return None;
    }

    Some(characters_on_page.iter().copied().collect())
}

pub fn get_all_characters_on_page_by_index(
    screenplay_document: &crate::screenplay_document::ScreenplayDocument,
    page_index: usize,
) -> Option<Vec<&screenplay_document::Character>> {
    let all_characters = screenplay_document.characters.iter().collect();
    //let characters_as_vec = all_characters
    filter_characters_by_page_index(&screenplay_document, all_characters, page_index)
}

// TODO: This function is wrong and bad???
// the test in the lib.rs just filters the scenes by location
// and then calls get_characters_for_scene,
// so that's what this function should do then...
pub fn filter_characters_by_location<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    characters_to_filter: Vec<&'a screenplay_document::Character>,
    location_id: &'a screenplay_document::LocationID,
) -> Option<Vec<&'a screenplay_document::Character>> {
    let mut characters_at_this_location: HashSet<&screenplay_document::Character> = HashSet::new();
    let Some(scenes_with_location) = get_scenes_with_location(screenplay_document, location_id)
    else {
        return None;
    };
    for (scene_id, _) in scenes_with_location {
        let Some(new_character_matches) =
            filter_characters_by_scene(screenplay_document, &characters_to_filter, scene_id)
        else {
            continue;
        };
        for character_match in new_character_matches {
            if characters_to_filter.contains(&character_match) {
                characters_at_this_location.insert(character_match);
            }
        }
    }
    if characters_at_this_location.is_empty() {
        return None;
    }
    Some(characters_at_this_location.iter().copied().collect())
}

pub fn get_characters_for_location<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    location_id: &'a screenplay_document::LocationID,
) -> Option<Vec<&'a screenplay_document::Character>> {
    let all_characters = screenplay_document.characters.iter().collect();
    let Some(thing) = filter_characters_by_location(
        screenplay_document,
        all_characters,
        location_id,
    ) else {
        return None;
    };
    if thing.is_empty() {
        return None;
    };
    Some(thing)
}

// ------------ Get SCENES...
// TODO: FILTER scenes by character speaking...

pub fn filter_scenes_by_locations<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    scenes_to_filter: Vec<(
        &'a screenplay_document::SceneID,
        &'a screenplay_document::Scene,
    )>,
    locations: Vec<&'a screenplay_document::LocationID>,
) -> Option<
    Vec<(
        &'a screenplay_document::SceneID,
        &'a screenplay_document::Scene,
    )>,
> {
    let mut filtered: Vec<(&screenplay_document::SceneID, &screenplay_document::Scene)> =
        Vec::new();
    for loc in locations {
        let Some(scenes_for_loc) = get_scenes_with_location(screenplay_document, loc) else {
            return None; // Could not find ANY scenes with this location...
        };

        for (scene, _) in scenes_for_loc {
            //println!("----- FILTERING BY LOCATION-SCENE....");
            for (scn_id, scn) in &filtered {
                if *scn_id == scene {
                    continue;
                }
            }
            for (id, scn) in &scenes_to_filter {
                if *id == scene {
                    filtered.push((id, scn));
                }
            }
        }
    }
    if filtered.is_empty() {
        return None;
    }
    Some(filtered)
}

/// Gets a Vec of all Scenes s in the document, sorted by document order.
///
/// Returns a Vec<(&SceneID, &Scene)>.
///
/// # Example
///
/// ```rust
/// use screenplay_doc_parser_rs::screenplay_document::*;
/// let mut doc = ScreenplayDocument::new();
/// let scene1: Scene = Scene {
///     start: ScreenplayCoordinate {page: 0 as usize, line: 10 as usize, element: None},
///     number: None,
///     revised: false,
///     environment: Environment::Int,
///     story_location: Location::default(),
///     story_sublocation: None,
///     story_time_of_day: None
/// };
/// let scene2: Scene = Scene {
///     start: ScreenplayCoordinate {page: 1 as usize, line: 5 as usize, element: None},
///     number: None,
///     revised: false,
///     environment: Environment::Int,
///     story_location: Location::default(),
///     story_sublocation: None,
///     story_time_of_day: None
/// };
/// let id_1 = SceneID::new();
/// let id_2 = SceneID::new();
/// doc.scenes.insert(id_2.clone(), scene2);
/// doc.scenes.insert(id_1.clone(), scene1);
///
/// let sorted = doc.get_all_scenes_sorted().unwrap();
///
/// assert_eq!(**sorted.get(0).unwrap(), id_1);
/// assert_eq!(**sorted.get(1).unwrap(), id_2)
/// ```
///
pub fn get_all_scenes_ordered<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
) -> Option<
    Vec<(
        &'a screenplay_document::SceneID,
        &'a screenplay_document::Scene,
    )>,
> {
    if screenplay_document.scenes.len() == 0 {
        return None;
    }
    if screenplay_document.scenes.len() == 1 {
        return Some(
            screenplay_document
                .scenes
                .iter()
                .map(|(id, scn)| (id, scn))
                .collect(),
        );
    }
    let mut all_scenes: Vec<_> = screenplay_document
        .scenes
        .iter()
        .map(|(a, b)| (a, b))
        .collect();

    all_scenes.sort_by(|(a_id, a_scn), (b_id, b_scn)| {
        (a_scn.start.page, a_scn.start.line).cmp(&(b_scn.start.page, b_scn.start.line))
    });

    return Some(all_scenes);
}

pub fn get_scene_for_screenplay_coordinate<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    checked_coordinate: &screenplay_document::ScreenplayCoordinate,
) -> Option<(
    &'a screenplay_document::SceneID,
    &'a screenplay_document::Scene,
)> {
    let Some(page) = screenplay_document.pages.get(checked_coordinate.page) else {
        //println!("DUCK SAUCE 2: HELP!");
        return None;
    };

    // lines are properly enumerated FORWARDS, then ENTIRE ITERATOR is reversed
    for (line_index, line) in page.lines.iter().enumerate().rev() {
        // if this line is EQUAL or EARLIER THAN the coordinate...
        if line_index > checked_coordinate.line {
            continue;
        }
        let Some(SPType::SP_SCENE_HEADING(screenplay_document::SceneHeadingElement::Line)) =
            line.line_type
        else {
            continue;
        };
        let Some(scn_id) = &line.scene_id else {
            //println!("SCENE HEADING HAS NO SCENE ID!");
            //panic!();
            continue;
        };
        //println!("SUCCESS PART ONE...?");
        let Some(scene_obj) = screenplay_document.scenes.get(scn_id) else {
            return None; // ERROR: BAD SCENE ID!
        };
        return Some((scn_id, scene_obj));
    }
    // couldn't find the scene on this page. try the previous page...
    // recursively check all previous pages

    let Some(previous_page_idx) = checked_coordinate.page.checked_sub(1) else {
        return None;
    };
    let Some(previous_page) = screenplay_document.pages.get(previous_page_idx) else {
        return None;
    };
    let Some(last_line_idx) = previous_page.lines.len().checked_sub(1) else {
        return None;
    };
    let last_page_last_line_coord = screenplay_document::ScreenplayCoordinate {
        page: previous_page_idx,
        line: last_line_idx,
        element: None,
    };
    //println!("CHECKING RECURSIVELY FOR SCENE!");
    //println!("COORDS OF PREVIOUS_PAGE: {:?}", last_page_last_line_coord);

    let Some(scene) =
        get_scene_for_screenplay_coordinate(screenplay_document, &last_page_last_line_coord)
    else {
        //println!("RECURSIVE CHECK FAILED!");
        //panic!("DUCK SAUCE");
        return None;
    };
    //println!("SUCCESS PART TWO...?");
    return Some(scene);
}

// This may be a contender for optimization....
pub fn get_scenes_from_range<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    start: &screenplay_document::ScreenplayCoordinate,
    end: &screenplay_document::ScreenplayCoordinate,
) -> Option<
    Vec<(
        &'a screenplay_document::SceneID,
        &'a screenplay_document::Scene,
    )>,
> {
    if (screenplay_document.pages.get(start.page)).is_none() {
        return None;
    }
    if screenplay_document.pages.get(end.page).is_none() {
        return None;
    }

    let mut scenes_in_range: Vec<(&screenplay_document::SceneID, &screenplay_document::Scene)> =
        Vec::new();
    for page_index in start.page..=end.page {
        let Some(page) = screenplay_document.pages.get(page_index) else {
            continue;
        };

        for (l_idx, line) in page.lines.iter().enumerate() {
            if page_index == start.page && l_idx < start.line {
                continue;
            } else if page_index == end.page && l_idx > end.line {
                break;
            }
            // TODO: keep track of CURRENT line type and PREVIOUS linetype
            // if PREV line was a SCENE HEADING, just continue
            // if CURRENT LINE is NOT a scene heading, AND we already HAVE an entry in the scenes vec, CONTINUE
            if Some(SPType::SP_SCENE_HEADING(
                screenplay_document::SceneHeadingElement::Line,
            )) != line.line_type
                && !scenes_in_range.is_empty()
            {
                continue;
            }
            let Some((scene_id, scene)) = get_scene_for_screenplay_coordinate(
                screenplay_document,
                &screenplay_document::ScreenplayCoordinate {
                    page: page_index,
                    line: l_idx,
                    element: None,
                },
            ) else {
                continue;
            };
            for (scn_id, scn) in &scenes_in_range {
                if scene_id == *scn_id {
                    continue;
                }
            }

            scenes_in_range.push((scene_id, scene));
        }
    }
    if scenes_in_range.is_empty() {
        return None;
    }

    Some(scenes_in_range)
}

pub fn filter_scenes_by_page_index<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    scenes_to_filter: Vec<(
        &'a screenplay_document::SceneID,
        &'a screenplay_document::Scene,
    )>,
    page_index: usize,
) -> Option<
    Vec<(
        &'a screenplay_document::SceneID,
        &'a screenplay_document::Scene,
    )>,
> {
    let page = screenplay_document.pages.get(page_index)?;
    if page.lines.is_empty() {
        return None;
    }
    let last_line_idx = page.lines.len().checked_sub(1).unwrap_or(0);
    let start = screenplay_document::ScreenplayCoordinate {
        page: page_index,
        line: 0,
        element: None,
    };
    let end = screenplay_document::ScreenplayCoordinate {
        page: page_index,
        line: last_line_idx,
        element: None,
    };
    let Some(scenes_on_page) = get_scenes_from_range(screenplay_document, &start, &end) else {
        return None;
    };

    let filtered: Vec<_> = scenes_on_page
        .iter()
        .filter(|s| scenes_to_filter.contains(&s))
        .copied()
        .collect();

    Some(filtered)
}

pub fn get_all_scenes_on_page_by_index(
    screenplay_document: &crate::screenplay_document::ScreenplayDocument,
    page_index: usize,
) -> Option<Vec<(&screenplay_document::SceneID, &screenplay_document::Scene)>> {
    let Some(all_scenes_ordered) = get_all_scenes_ordered(screenplay_document) else {
        return None;
    };
    filter_scenes_by_page_index(screenplay_document, all_scenes_ordered, page_index)
}

// this one seems inefficient....
// could skip lines of loop if we already pushed the current valid scene, and skip
// to the first line of the next scene to check each time...
pub fn get_all_scenes_with_character_speaking<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    character: &screenplay_document::Character,
) -> Option<
    Vec<(
        &'a screenplay_document::SceneID,
        &'a screenplay_document::Scene,
    )>,
> {
    let all_scenes_sorted = get_all_scenes_ordered(screenplay_document)?;
    return filter_scenes_by_character_speaking(screenplay_document, all_scenes_sorted, character);
}

pub fn filter_scenes_by_character_speaking<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    mut scenes_to_filter: Vec<(
        &'a screenplay_document::SceneID,
        &'a screenplay_document::Scene,
    )>,
    character: &screenplay_document::Character,
) -> Option<
    Vec<(
        &'a screenplay_document::SceneID,
        &'a screenplay_document::Scene,
    )>,
> {
    if scenes_to_filter.is_empty() {
        return None;
    }
    scenes_to_filter.sort_by(|(_a, b), (_c, d)| {
        (b.start.page, b.start.line).cmp(&(d.start.page, d.start.line))
    });

    let mut scenes_vec: Vec<_> = Vec::new();

    // COULD OPTIMIZE
    // this just re-starts iteration for ALL PAGES every time
    // ... I tried to optimize it but I just broke it LMAO I'm gonna leave it as is...
    'scenes: for (scn_id, scn) in scenes_to_filter {
        'pages: for (p_idx, page) in screenplay_document.pages.iter().enumerate() {
            'lines: for (l_idx, line) in page.lines.iter().enumerate() {
                if p_idx < scn.start.page {
                    continue 'pages;
                }
                if l_idx < scn.start.line {
                    continue 'lines;
                }
                if line.line_type
                    == Some(SPType::SP_SCENE_HEADING(
                        screenplay_document::SceneHeadingElement::Line,
                    ))
                    && line.scene_id != Some(*scn_id)
                {
                    continue 'scenes;
                }
                if character.is_line(&line) {
                    for (existing_id, _) in &scenes_vec {
                        if *existing_id == scn_id {
                            continue 'lines;
                        }
                    }

                    scenes_vec.push((scn_id, scn));
                }
            }
        }
    }
    if scenes_vec.is_empty() {
        return None;
    } else {
        return Some(scenes_vec);
    }
}

pub fn get_scenes_with_location<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    location: &'a screenplay_document::LocationID,
) -> Option<
    Vec<(
        &'a screenplay_document::SceneID,
        &'a screenplay_document::Scene,
    )>,
> {
    let scenes: Vec<_> = screenplay_document
        .scenes
        .iter()
        .filter(|(_, scn)| scn.story_locations.contains(&location))
        .collect();

    if scenes.is_empty() {
        let location_opt = screenplay_document.locations.get(&location);
        println!("LOCATION: {:?}", location_opt);
        //panic!("COULDN'T FIND SCENES FOR LOCATION!");
        return None;
    }

    Some(scenes)
}

// ------------ Get PAGEs...
// TODO: Filter pages by location, character

pub fn get_all_pages_for_scene<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    scene_id: &'a screenplay_document::SceneID,
) -> Option<Vec<(usize, &'a screenplay_document::Page)>> {
    let checked_scene = screenplay_document.scenes.get(scene_id)?;

    let scenes_ordered = get_all_scenes_ordered(screenplay_document)?;
    //pages.push(checked_scene.start.page.clone());

    for (_this_id, scene_obj) in scenes_ordered {
        if scene_obj.start.page < checked_scene.start.page
            || (scene_obj.start.page == checked_scene.start.page
                && scene_obj.start.line < checked_scene.start.line)
        {
            continue;
        }
        let page_indecies: Vec<usize> =
            (checked_scene.start.page.clone()..=scene_obj.start.page.clone()).collect();

        let mut pages: Vec<(usize, &screenplay_document::Page)> = Vec::new();
        for pi in page_indecies {
            let Some(page) = screenplay_document.pages.get(pi) else {
                continue;
            };
            pages.push((pi.clone(), page));
        }
        if pages.is_empty() {
            return None;
        }
        return Some(pages);
    }

    None
}

pub fn get_all_pages_for_multiple_scenes<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    scene_ids: Vec<(&screenplay_document::SceneID, &screenplay_document::Scene)>,
) -> Option<Vec<(usize, &'a screenplay_document::Page)>> {
    let scenes_ordered = get_all_scenes_ordered(screenplay_document)?;

    let mut all_pages: Vec<(usize, &screenplay_document::Page)> = Vec::new();

    for (scene_id, checked_scene) in scene_ids {
        //pages.push(checked_scene.start.page.clone());

        for (_this_id, scene_obj) in &scenes_ordered {
            if scene_obj.start.page < checked_scene.start.page
                || (scene_obj.start.page == checked_scene.start.page
                    && scene_obj.start.line < checked_scene.start.line)
            {
                continue;
            }
            let page_indecies: Vec<usize> =
                (checked_scene.start.page.clone()..=scene_obj.start.page.clone()).collect();

            let mut pages: Vec<(usize, &screenplay_document::Page)> = Vec::new();
            for pi in page_indecies {
                let Some(page) = screenplay_document.pages.get(pi) else {
                    continue;
                };
                pages.push((pi.clone(), page));
            }
            if pages.is_empty() {
                continue;
            }
            for (idx, page) in pages {
                if !all_pages.contains(&(idx, page)) {
                    all_pages.push((idx, page));
                }
            }
        }
    }
    if all_pages.is_empty() {
        return None;
    }

    Some(all_pages)
}

pub fn get_all_pages_for_location<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    location: &'a screenplay_document::LocationID,
) -> Option<Vec<(usize, &'a screenplay_document::Page)>> {
    let Some(scenes_ordered) = get_all_scenes_ordered(screenplay_document) else {
        return None;
    };
    let scenes_filtered =
        filter_scenes_by_locations(screenplay_document, scenes_ordered, vec![location])?;

    return get_all_pages_for_multiple_scenes(screenplay_document, scenes_filtered);
}

pub fn get_all_pages_for_character_speaking<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    character: &'a screenplay_document::Character,
) -> Option<Vec<(usize, &'a screenplay_document::Page)>> {
    let mut pages: Vec<(usize, &screenplay_document::Page)> = Vec::new();

    'pages: for (idx, page) in screenplay_document.pages.iter().enumerate() {
        for ln in &page.lines {
            if character.is_line(ln) {
                pages.push((idx, page));
                continue 'pages;
            }
        }
    }

    if pages.is_empty() {
        return None;
    }

    Some(pages)
}

pub fn filter_pages_by_character_speaking<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    pages_range: (usize, usize),
    character: &'a screenplay_document::Character,
) -> Option<Vec<(usize, &'a screenplay_document::Page)>> {
    let mut filtered_pages: Vec<(usize, &screenplay_document::Page)> = Vec::new();

    'page_indices: for i in pages_range.0..=pages_range.1 {
        let Some(page) = screenplay_document.pages.get(i) else {
            continue;
        };

        for ln in &page.lines {
            if character.is_line(ln) {
                //panic!("WHAT?");
                filtered_pages.push((i, page));
                continue 'page_indices;
            }
        }
    }
    if filtered_pages.is_empty() {
        return None;
    }

    Some(filtered_pages)
}

pub fn filter_pages_by_multiple_characters_speaking<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    pages_range: (usize, usize),
    characters: Vec<&'a screenplay_document::Character>,
) -> Option<Vec<(usize, &'a screenplay_document::Page)>> {
    let mut filtered_pages: Vec<(usize, &screenplay_document::Page)> = Vec::new();

    'page_indices: for i in pages_range.0..=pages_range.1 {
        let Some(page) = screenplay_document.pages.get(i) else {
            continue;
        };

        for ln in &page.lines {
            for character in &characters {
                if character.is_line(ln) {
                    filtered_pages.push((i, page));
                    continue 'page_indices;
                }
            }
        }
    }
    if filtered_pages.is_empty() {
        return None;
    }

    Some(filtered_pages)
}

pub fn filter_pages_by_location<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    pages_range: (usize, usize),
    location: &'a screenplay_document::LocationID,
) -> Option<Vec<(usize, &'a screenplay_document::Page)>> {
    let Some(scenes_ordered) = get_all_scenes_ordered(screenplay_document) else {
        return None;
    };
    let scenes_within_page_range: Vec<(_)> = scenes_ordered
        .iter()
        .filter(|(scn_id, scn)| pages_range.0 < scn.start.page && scn.start.page < pages_range.1)
        .copied()
        .collect();
    let scenes_filtered = filter_scenes_by_locations(
        screenplay_document,
        scenes_within_page_range,
        vec![location],
    )?;

    return get_all_pages_for_multiple_scenes(screenplay_document, scenes_filtered);
}
pub fn filter_pages_by_multiple_locations<'a>(
    screenplay_document: &'a crate::screenplay_document::ScreenplayDocument,
    pages_range: (usize, usize),
    locations: Vec<&'a screenplay_document::LocationID>,
) -> Option<Vec<(usize, &'a screenplay_document::Page)>> {
    let Some(scenes_ordered) = get_all_scenes_ordered(screenplay_document) else {
        return None;
    };
    let scenes_within_page_range: Vec<(_)> = scenes_ordered
        .iter()
        .filter(|(scn_id, scn)| pages_range.0 < scn.start.page && scn.start.page < pages_range.1)
        .copied()
        .collect();
    let scenes_filtered =
        filter_scenes_by_locations(screenplay_document, scenes_within_page_range, locations)?;

    return get_all_pages_for_multiple_scenes(screenplay_document, scenes_filtered);
}
