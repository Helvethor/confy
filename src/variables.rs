use std::ops::Deref;
use std::cell::Cell;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Variables {
    map: HashMap<String, String>
}

#[derive(Copy, Clone)]
enum Node {
    Root(usize),
    Child(usize)
}

struct DisjointSet {
    elements: Vec<Cell<Node>>
}

impl Deref for Variables {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &HashMap<String, String> {
        &self.map
    }
}

impl Variables {
    pub fn new(map: &HashMap<String, String>) -> Variables {
        let original = map;
        let mut map = HashMap::with_capacity(original.len());

        let mut recto = HashMap::with_capacity(original.len());
        let mut verso = HashMap::with_capacity(original.len());

        for (i, key) in original.keys().enumerate() {
            recto.insert(&key[..], i);
            verso.insert(i, &key[..]);
        }

        let mut set = DisjointSet::new(original.len());

        for (i, value) in original.values().enumerate() {
            if value.starts_with("@") {
                if let Some(j) = recto.get(&value[1..]) {
                    set.merge(*j, i);
                }
            }
        }

        for key in original.keys() {
            let i = recto.get(&key[..]).unwrap();
            let j = set.root(*i);
            let deref_key = verso.get(&j).unwrap();
            let value = original.get(&deref_key[..]).unwrap();
            map.insert(key.clone(), value.clone());
        }

        Variables { map }
    }
}


impl DisjointSet {

    fn new(size: usize) -> DisjointSet {
        let mut elements = Vec::with_capacity(size);

        for _ in 0..size {
            elements.push(Cell::new(Node::Root(1)));
        }

        DisjointSet { elements }
    }

    #[allow(dead_code)]
    fn size(&self, n: usize) -> usize{
        let r = self.root(n);
        if let Node::Root(size) = self.elements[r].get() {
            size
        }
        else {
            panic!("Root not found for {}\n", n)
        }
    }

    fn root(&self, n: usize) -> usize {
        match self.elements[n].get() {
            Node::Root(_) => n,
            Node::Child(m) => {
                let r = self.root(m);
                self.elements[n].set(Node::Child(r));
                r
            }
        }
    }

    fn merge(&mut self, n: usize, m: usize) {
        let a = self.root(n);
        let b = self.root(m);

        if a == b {
            return
        }

        match self.elements[a].get_mut() {
            &mut Node::Root(mut _size_a) => {
                match self.elements[b].get() {
                    Node::Root(_size_b) => {
                        _size_a += _size_b;
                        self.elements[b].set(Node::Child(a));
                    },
                    Node::Child(r) =>
                        panic!("Unexepected child element: Child({})\n", r)
                }
            },
            &mut Node::Child(r) => 
                panic!("Unexepected child element: Child({})\n", r)
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_variables() {
        let mut map = HashMap::new();
        let abc = "ABC".to_string();

        map.insert("ab".to_string(), "@abc".to_string());
        map.insert("-a".to_string(), "@".to_string());
        map.insert("".to_string(), "@a".to_string());
        map.insert("a".to_string(), "@ab".to_string());
        map.insert("abc".to_string(), abc.clone());

        map.insert("loop0".to_string(), "@loop1".to_string());
        map.insert("loop1".to_string(), "@loop0".to_string());

        let variables = Variables::new(&map);

        assert_eq!(variables.get("-a").unwrap(), &abc);
        assert_eq!(variables.get("").unwrap(), &abc);
        assert_eq!(variables.get("a").unwrap(), &abc);
        assert_eq!(variables.get("ab").unwrap(), &abc);
        assert_eq!(variables.get("abc").unwrap(), &abc);

        assert_eq!(variables.get("loop0").unwrap(), variables.get("loop1").unwrap());
    }

    #[test]
    fn test_disjoint_set() {
        let mut set = DisjointSet::new(10);
        set.merge(0, 1);
        set.merge(1, 2);
        set.merge(2, 3);
        set.merge(4, 5);
        set.merge(6, 7);
        set.merge(8, 9);

        assert_ne!(set.root(0), set.root(9));
        assert_ne!(set.root(4), set.root(6));
        assert_ne!(set.root(8), set.root(3));

        assert_eq!(set.root(0), set.root(1));
        assert_eq!(set.root(0), set.root(2));
        assert_eq!(set.root(0), set.root(3));

        assert_eq!(set.root(4), set.root(5));

        assert_eq!(set.root(6), set.root(7));

        assert_eq!(set.root(8), set.root(9));

        set.merge(0, 9);

        assert_eq!(set.root(8), set.root(3));

    }
}
