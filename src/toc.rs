use crate::prelude::*;

use xhtml_elem::*;

pub struct TableOfContents {
    items: Vec<Box<dyn ToCItemTrait>>,
}

impl TableOfContents {
    pub fn to_xhtml(&self, min_level: u8, nav: &PathBuf) -> String {
        self.to_xhtml_elem(min_level, nav).to_html()
    }

    pub fn to_xhtml_elem(&self, min_level: u8, nav: &PathBuf) -> OL {
        let level = 0;

        let li_vec = self.items
            .iter()
            .map(|item| item.to_xhtml_elem(min_level, nav))
            .collect::<Vec<Box<dyn Elem>>>();

        OL {
            elems: li_vec,
            hidden: level >= min_level,
        }
    }

    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    pub fn push(&mut self, item: Box<dyn ToCItemTrait>) {
        let level = 0;

        if (level + 1) == item.level() {
            self.items.push(item);
        } else {
            match self.items.last_mut() {
                Some(last) => {
                    last.push(item);
                }
                None => {
                    let mut dummy = ToCDummyItem::new(level + 1);
                    dummy.push(item);
                    self.items.push(Box::new(dummy));
                }
            }
        }
    }

    pub fn size(&self) -> usize {
        let mut size = 0;
        for item in &self.items {
            size += item.size();
        }
        size
    }
}

pub trait ToCItemTrait {
    fn items(&self) -> &Vec<Box<dyn ToCItemTrait>>;

    fn items_mut(&mut self) -> &mut Vec<Box<dyn ToCItemTrait>>;

    fn level(&self) -> u8;

    fn is_dummy(&self) -> bool;

    fn push(&mut self, item: Box<dyn ToCItemTrait>) {
        let level = self.level();

        if (level + 1) == item.level() {
            self.items_mut().push(item);
        } else {
            match self.items_mut().last_mut() {
                Some(last) => {
                    last.push(item);
                }
                None => {
                    let mut dummy = ToCDummyItem::new(level + 1);
                    dummy.push(item);
                    self.items_mut().push(Box::new(dummy));
                }
            }
        }
    }

    fn to_a(&self, navigation_path: &PathBuf) -> Option<A>;

    fn to_xhtml_elem(&self, min_level: u8, navigation_path: &PathBuf) -> Box<dyn Elem> {
        let mut li = LI {
            elems: Vec::new(),
        };

        let a = self.to_a(navigation_path);
        if let Some(a) = a {
            li.push(Box::new(a));
        }

        let li_vec
            = self.items()
            .iter()
            .map(|item| item.to_xhtml_elem(min_level, navigation_path))
            .collect::<Vec<Box<dyn Elem>>>();

        let ol = OL { elems: li_vec, hidden: self.level() >= min_level };
        li.push(Box::new(ol));

        Box::new(li)
    }

    fn size(&self) -> usize {
        let mut size = 0;
        for item in self.items() {
            size += item.size();
        }
        size + 1
    }
}

pub struct ToCItem {
    pub items: Vec<Box<dyn ToCItemTrait>>,
    pub path_buf: PathBuf,
    pub id: Option<String>,
    pub title: String,
    pub level: u8,
}

impl ToCItemTrait for ToCItem {
    fn items(&self) -> &Vec<Box<dyn ToCItemTrait>> {
        self.items.as_ref()
    }

    fn items_mut(&mut self) -> &mut Vec<Box<dyn ToCItemTrait>> {
        self.items.as_mut()
    }

    fn level(&self) -> u8 {
        self.level
    }

    fn is_dummy(&self) -> bool {
        false
    }

    fn to_a(&self, navigation_path: &PathBuf) -> Option<A> {
        Some(A {
            text: self.title.clone(),
            href: {
                let path
                    = PathBuf::path_diff(navigation_path, &self.path_buf)
                    .map(|p| {
                        p.to_str()
                            .map(|s| s.to_string())
                            .unwrap_or_default()
                    }).unwrap_or_default();
                let id = self.id.clone().unwrap_or_default();

                format!("{}#{}", &path, &id)
            },
        })
    }
}

pub struct ToCDummyItem {
    items: Vec<Box<dyn ToCItemTrait>>,
    level: u8,
}

impl ToCDummyItem {
    pub fn new(level: u8) -> Self {
        Self {
            items: Vec::new(),
            level,
        }
    }
}

impl ToCItemTrait for ToCDummyItem {
    fn items(&self) -> &Vec<Box<dyn ToCItemTrait>> {
        self.items.as_ref()
    }

    fn items_mut(&mut self) -> &mut Vec<Box<dyn ToCItemTrait>> {
        self.items.as_mut()
    }

    fn level(&self) -> u8 {
        self.level
    }

    fn is_dummy(&self) -> bool {
        true
    }

    #[allow(unused_variables)]
    fn to_a(&self, navigation_path: &PathBuf) -> Option<A> {
        None
    }
}

mod xhtml_elem {
    pub trait Elem {
        fn elems(&self) -> Option<&Vec<Box<dyn Elem>>>;
        fn to_html(&self) -> String;
        fn push(&mut self, elem: Box<dyn Elem>);
    }

    pub struct OL {
        pub elems: Vec<Box<dyn Elem>>,
        pub hidden: bool,
    }

    impl Elem for OL {
        fn elems(&self) -> Option<&Vec<Box<dyn Elem>>> {
            Some(self.elems.as_ref())
        }

        fn to_html(&self) -> String {
            let inner_text
                = self.elems
                .iter()
                .map(|elem| {
                    elem.to_html()
                }).collect::<Vec<String>>().join("\n");
            if self.elems.len() == 0 {
                format!("")
            } else if self.hidden {
                format!("<ol hidden=\"hidden\">\n{}\n</ol>", &inner_text)
            } else {
                format!("<ol>\n{}\n</ol>", &inner_text)
            }
        }

        fn push(&mut self, elem: Box<dyn Elem>) {
            self.elems.push(elem);
        }
    }

    pub struct LI {
        pub elems: Vec<Box<dyn Elem>>,
    }

    impl Elem for LI {
        fn elems(&self) -> Option<&Vec<Box<dyn Elem>>> {
            Some(self.elems.as_ref())
        }

        fn to_html(&self) -> String {
            let inner_text
                = self.elems
                .iter()
                .map(|elem| {
                    elem.to_html()
                }).collect::<Vec<String>>().join("\n");
            format!("<li>\n{}\n</li>", &inner_text)
        }

        fn push(&mut self, elem: Box<dyn Elem>) {
            self.elems.push(elem);
        }
    }

    pub struct A {
        pub href: String,
        pub text: String,
    }

    impl Elem for A {
        fn elems(&self) -> Option<&Vec<Box<dyn Elem>>> {
            None
        }

        fn to_html(&self) -> String {
            format!("<a href=\"{}\">{}</a>", &self.href, &self.text)
        }

        #[allow(unused_variables)]
        fn push(&mut self, elem: Box<dyn Elem>) {
            unimplemented!()
        }
    }

    #[test]
    fn test() {
        use super::*;

        let mut ol = OL {
            elems: Vec::new(),
            hidden: false,
        };
        for _ in 0..3 {
            let mut li = LI {
                elems: Vec::new(),
            };

            let href = String::from_str("ex/am/ple").unwrap();
            let text = String::from_str("SampleTitle").unwrap();
            let a = A {
                href,
                text,
            };
            li.push(Box::new(a));
            ol.push(Box::new(li));
        }

        println!("{}", ol.to_html());
        assert_eq!(1, 1)
    }
}

/// execute with --nocapture option
#[test]
fn test() {
    let mut toc = TableOfContents::new();

    for _ in 0..5 {
        let tocc = ToCItem {
            items: Vec::new(),
            path_buf: PathBuf::from_str("ex/am/ple").unwrap(),
            id: Some(String::from_str("sample id").unwrap()),
            title: String::from_str("sample title").unwrap(),
            level: 1,
        };
        toc.push(Box::new(tocc));
    }

    println!("{}", toc.to_xhtml(2, &PathBuf::from_str("ex").unwrap()));

    assert_eq!(1, 1)
}