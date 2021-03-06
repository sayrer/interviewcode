#![cfg_attr(test, feature(test))]

pub static ASCII_TEXT: &'static str = "Attend to hear 6 stellar #mobile #startups at #OF12 Entrepreneur Idol show 2day,  http://t.co/HtzEMgAC @TiEcon @sv_entrepreneur @500!";
pub static UNICODE_TEXT: &'static str = "Attend \u{20000}\u{20000} hear 6 stellar #mobile #startups at #OF12 Entrepreneur Idol show 2day,  http://t.co/HtzEMgAC @TiEcon @sv_entrepreneur @500!";

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct Entity {
    start: usize,
    end: usize,
    html: String
}

impl Ord for Entity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}
impl PartialOrd for Entity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone)]
pub struct DecodedEntity {
    start: usize,
    end: usize,
    html: Vec<char>,
}

impl Entity {
    fn decode(&self) -> DecodedEntity {
        DecodedEntity {start: self.start, end: self.end, html: self.html.chars().collect()}
    }
}

fn render(text: &str, entities: &Vec<Entity>) -> String {
    let mut sb = String::with_capacity(text.len()*2);
    let mut my_entities = entities.clone();
    my_entities.sort_by(|e1, e2| e1.start.cmp(&e2.start) );

    let mut pos = 0 as usize;
    for entity in my_entities {
        sb.extend(text.chars().skip(pos).take(entity.start - pos));
        sb.push_str(entity.html.as_str());
        pos = entity.end;
    }
    for c in text.chars().skip(pos).take(text.chars().count() - pos) {
        sb.push(c);
    }
    sb
}

fn render_chars(text: &Vec<char>, entities: &Vec<DecodedEntity>) -> String {
    let mut sb: Vec<char> = Vec::with_capacity(text.len()*2);
    let mut my_entities = entities.clone();
    my_entities.sort_by(|e1, e2| e1.start.cmp(&e2.start) );

    let mut pos = 0 as usize;
    for entity in my_entities {
        sb.extend_from_slice(&text[pos..entity.start]);
        sb.extend_from_slice(&entity.html);
        pos = entity.end;
    }
    sb.extend_from_slice(&text[pos..text.len()]);
    sb.into_iter().collect()
}

fn render_chars2(text: &Vec<char>, entities: &Vec<Entity>) -> String {
    let mut my_entities = entities.clone();
    my_entities.sort();
    let mut sb = String::with_capacity(text.len()*2);
    let mut pos = 0 as usize;
    for entity in my_entities {
        for i in pos..entity.start {
            sb.push(text[i]);
        }
        sb.push_str(&entity.html);
        pos = entity.end;
    }
    for i in pos..text.len() {
        sb.push(text[i]);
    }
    sb
}

#[derive(Debug, Copy, Clone)]
struct EntityIndex {
    index: usize,
    start: usize
}

fn render_chars_stack(text: &Vec<char>, entities: &Vec<Entity>) -> String {
    // sort the entities on the stack
    let mut enitity_indices: [EntityIndex; 280] = [EntityIndex{index: 10000, start:10000}; 280];
    for (i, entity) in entities.iter().enumerate() {
        if i == 0 {
            enitity_indices[0] = EntityIndex{index: 0, start: entity.start};
        } else {
            for index in 0..i+1 {
                if enitity_indices[index].start > entity.start {
                    for mov in (index + 1 .. i+1).rev() {
                        enitity_indices[mov] = enitity_indices[mov - 1];
                    }
                    enitity_indices[index] = EntityIndex{index: i, start:entity.start};
                    break;
                }
            }
        }
    }

    let mut sb = String::with_capacity(text.len()*2);
    let mut pos = 0 as usize;
    for i in 0..entities.len() {
        let entity = &entities[enitity_indices[i].index];
        for i in pos..entity.start {
            sb.push(text[i]);
        }
        sb.push_str(&entity.html);
        pos = entity.end;
    }
    for i in pos..text.len() {
        sb.push(text[i]);
    }
    sb
}

fn main() {
    let result = render(&ASCII_TEXT, &mut entities());
    println!("Result: {}", result);
}

pub fn entities() -> Vec<Entity> {
    let entities = vec![
    Entity {start: 82, end: 102, html:"<http://t.co/HtzEMgAC>".to_string()},
    Entity {start: 128, end: 132, html:"<@500>".to_string()},
    Entity {start: 25, end: 32, html:"<#mobile>".to_string()},
    Entity {start: 33, end: 42, html:"<#startups>".to_string()},
    Entity {start: 111, end: 127, html:"<@sv_entrepreneur>".to_string()},
    Entity {start: 46, end: 51, html:"<#OF12>".to_string()},
    Entity {start: 103, end: 110, html:"<@TiEcon>".to_string()},
    ];
    entities
}

pub fn decoded_entities() -> Vec<DecodedEntity> {
    entities().into_iter().map( |e| e.decode() ).collect()
}


#[cfg(test)] extern crate rand;
#[cfg(test)] extern crate test;

use std::cmp::Ordering;

#[cfg(test)]
mod rendertest {
    use super::*;
    use rand::{self,Rng};
    use test::Bencher;

    fn generate_entities() -> Vec<Vec<Entity>> {
        let mut rng = rand::thread_rng();
        let mut entities_list: Vec<Vec<Entity>> = Vec::with_capacity(1000);

        for _ in 0..1000 {
            let total = rng.gen::<usize>() % 10;
            let mut indices = Vec::with_capacity(total);
            for _ in 0..(total*2) {
                loop {
                    let index = rng.gen::<usize>() % ASCII_TEXT.len();
                    if !indices.contains(&index) {
                        indices.push(index);
                        break;
                    }
                }
            }

            indices.sort();
            let entities = indices.chunks(2).map(|chunk| {
                let (start, end) = (chunk[0], chunk[1]);
                let length = end - start;
                Entity {start: start, end: end, html: (0..length).map(|_| "X").collect()}
            });
            entities_list.push(entities.collect());
        }

        entities_list
    }

    fn generate_decoded_entities() -> Vec<Vec<DecodedEntity>> {
        generate_entities().into_iter().map(|entries| {
            entries.into_iter().map(|e| { e.decode() } ).collect()
        }).collect()
    }

    #[test]
    fn correctness_chars() {
        let result = "Attend \u{20000}\u{20000} hear 6 stellar <#mobile> <#startups> at <#OF12> Entrepreneur Idol show 2day,  <http://t.co/HtzEMgAC> <@TiEcon> <@sv_entrepreneur> <@500>!";
        assert_eq!(result, render_chars(&UNICODE_TEXT.chars().collect(), &decoded_entities()))
    }

    #[test]
    fn correctness() {
        let result = "Attend \u{20000}\u{20000} hear 6 stellar <#mobile> <#startups> at <#OF12> Entrepreneur Idol show 2day,  <http://t.co/HtzEMgAC> <@TiEcon> <@sv_entrepreneur> <@500>!";
        assert_eq!(result, render(&UNICODE_TEXT, &entities()))
    }

    #[test]
    fn correctness_chars2() {
        let result = "Attend \u{20000}\u{20000} hear 6 stellar <#mobile> <#startups> at <#OF12> Entrepreneur Idol show 2day,  <http://t.co/HtzEMgAC> <@TiEcon> <@sv_entrepreneur> <@500>!";
        assert_eq!(result, render_chars2(&UNICODE_TEXT.chars().collect(), &entities()))
    }

    #[test]
    fn correctness_chars_stack() {
        let result = "Attend \u{20000}\u{20000} hear 6 stellar <#mobile> <#startups> at <#OF12> Entrepreneur Idol show 2day,  <http://t.co/HtzEMgAC> <@TiEcon> <@sv_entrepreneur> <@500>!";
        assert_eq!(result, render_chars_stack(&UNICODE_TEXT.chars().collect(), &entities()))
    }

    #[bench]
    fn bench_replacement(b: &mut Bencher) {
        let entities_list = generate_entities();
        let mut index_iter = (0..1000).into_iter().cycle();
        b.iter(|| {
            render(UNICODE_TEXT, &entities_list[index_iter.next().unwrap()])
        });
    }

    #[bench]
    fn bench_replacement_chars(b: &mut Bencher) {
        let entities_list = generate_decoded_entities();
        let mut index_iter = (0..1000).into_iter().cycle();
        let decoded_text = UNICODE_TEXT.chars().collect();
        b.iter(|| {
            let option = index_iter.next();
            render_chars(&decoded_text, &entities_list[option.unwrap()].clone())
        });
    }

    #[bench]
    fn bench_replacement_chars2(b: &mut Bencher) {
        let entities_list = generate_entities();
        let mut index_iter = (0..1000).into_iter().cycle();
        let decoded_text = UNICODE_TEXT.chars().collect();
        b.iter(|| {
            let option = index_iter.next();
            render_chars2(&decoded_text, &entities_list[option.unwrap()])
        });
    }

    #[bench]
    fn bench_replacement_chars_stack(b: &mut Bencher) {
        let entities_list = generate_entities();
        let mut index_iter = (0..1000).into_iter().cycle();
        let decoded_text = UNICODE_TEXT.chars().collect();
        b.iter(|| {
            let option = index_iter.next();
            render_chars_stack(&decoded_text, &entities_list[option.unwrap()])
        });
    }
}
