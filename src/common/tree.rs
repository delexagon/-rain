use std::collections::HashMap;
use super::RemoveVec;

pub struct ExtTree<D> (pub D, pub Vec<ExtTree<D>>);
impl<D> ExtTree<D> {
    pub fn recursive_map<T, F>(self, f: &F) -> ExtTree<T> where F: Fn(D) -> T {
        let ExtTree(dat, mut vec) = self;
        return ExtTree(f(dat), vec.drain(..).map(|x| x.recursive_map(f)).collect());
    }
}
pub type RecTree<D> = ExtTree<(bool, D)>;


impl<'a, Dat> From<(&'a mut Vec<Id>, RecTree<Dat>)> for Tree<Dat> {
    fn from((record, tree): (&'a mut Vec<Id>, RecTree<Dat>)) -> Tree<Dat> {
        let ExtTree((keep, obj), children) = tree;
        let mut me = Tree::new(obj);
        let root = me.root();
        if keep {record.push(root)}
        for child in children {
            me.comprehend_record(root, child, record);
        }
        return me;
    }
}

impl<Dat> From<ExtTree<Dat>> for Tree<Dat> {
    fn from(tree: ExtTree<Dat>) -> Tree<Dat> {
        let ExtTree(obj, children) = tree;
        let mut me = Tree::new(obj);
        let root = me.root();
        for child in children {
            me.comprehend(root, child);
        }
        return me;
    }
}



/// A missing edge in the tree, which
#[must_use]
pub struct MissingEdge<'a, Dat>(&'a mut Tree<Dat>, Edge);
impl<'a, Dat> MissingEdge<'a, Dat> {
    /// Shifts every child past the edge left,
    /// removing the empty space
    pub fn collapse(self) -> &'a mut Tree<Dat> {
        let MissingEdge(tree, Edge(parent, empty)) = self;
        // It is a runtime error to attempt to collapse the root node.
        let parent = parent.unwrap().0;
        let children = tree.dat[parent].children;
        for child in empty..children-1 {
            let old_edge = Edge(Some(Id(parent)),child+1);
            let new_edge = Edge(Some(Id(parent)),child);
            let child_id = tree.child_map[&old_edge];
            tree.dat[child_id.0].parent = new_edge;
            tree.child_map.insert(new_edge, child_id);
        }
        tree.child_map.remove(&Edge(Some(Id(parent)), children-1));
        tree.dat[parent].children -= 1;
        return tree;
    }
    /// 
    pub fn add_node(self, obj: Dat) -> Id {
        let MissingEdge(tree, edge) = self;
        let child_id = Id(tree.dat.push(Node {obj, parent: edge, children: 0}));
        tree.child_map.insert(edge,child_id);
        return child_id;
    }
    pub fn add_exttree(self, val: ExtTree<Dat>) -> Id {
        let MissingEdge(this_tree, edge) = self;
        let ExtTree(obj, children) = val;
        let top_id = Id(this_tree.dat.push(Node {obj, parent: edge, children: 0}));
        this_tree.child_map.insert(edge,top_id);
        for child in children {
            this_tree.comprehend(top_id, child);
        }
        return top_id;
    }
    pub fn add_rectree(self, (record, tree): (&'a mut Vec<Id>, RecTree<Dat>)) -> Id {
        let MissingEdge(this_tree, edge) = self;
        let ExtTree((keep, obj), children) = tree;
        let top_id = Id(this_tree.dat.push(Node {obj, parent: edge, children: 0}));
        this_tree.child_map.insert(edge,top_id);
        if keep {record.push(top_id)}
        for child in children {
            this_tree.comprehend_record(top_id, child, record);
        }
        return top_id;
    }
    pub fn add_tree(self, mut other: Tree<Dat>) -> Id {
        let MissingEdge(tree, edge) = self;
        let other_root = other.root();
        let Node {children, obj, ..} = other.dat.remove(other_root.0).unwrap();
        let base = Id(tree.dat.push(Node {obj, parent: edge, children}));
        tree.child_map.insert(edge, base);
        for child in 0..children {
            let other_child = other.child_map[&Edge(Some(other_root), child)];
            let new_child = tree.comprehend_other(&mut other, Edge(Some(base), child), other_child);
            tree.child_map.insert(Edge(Some(base), child), new_child);
        }
        return base;
    }
    pub fn transfer(self, other: LooseTree<'_, Dat>) -> Id {
        let MissingEdge(tree, edge) = self;
        let LooseTree(other_tree, loose_root) = other;
        let Node {children, obj, ..} = other_tree.dat.remove(loose_root.0).unwrap();
        let base = Id(tree.dat.push(Node {obj, parent: edge, children}));
        tree.child_map.insert(edge, base);
        for child in 0..children {
            let other_child = other_tree.child_map[&Edge(Some(loose_root), child)];
            let new_child = tree.comprehend_other(other_tree, Edge(Some(base), child), other_child);
            tree.child_map.insert(Edge(Some(base), child), new_child);
        }
        return base;
        
    }
    /// This function must only be used when other is not an ancestor of the missing edge.  
    /// If this is not the case, the tree will be left in a broken state.
    pub fn swap(self, other: Id) -> MissingEdge<'a,Dat> {
        let MissingEdge(tree, new_edge) = self;
        let other_edge = tree.dat[other.0].parent;
        tree.child_map.remove(&other_edge);
        tree.dat[other.0].parent = new_edge;
        tree.child_map.insert(new_edge, other);
        return MissingEdge(tree, other_edge);
    }
    pub fn cut_ancestor(self, ancestor: Id) -> MissingAndLooseWithMissing<'a,Dat> {
        let MissingEdge(tree, loose_edge) = self;
        let MissingAndLoose(tree, ancestor_edge, loose_tree) = tree.cut(ancestor);
        return MissingAndLooseWithMissing(tree, ancestor_edge, loose_tree, loose_edge);
    }
    /// Swaps with another child of this same parent, if it exists.
    pub fn reorder(self, index: i32) -> MissingEdge<'a,Dat> {
        let MissingEdge(tree, maybe_parent) = self;
        match maybe_parent {
            Edge(Some(parent), child1) => {
                let children = tree.dat[parent.0].children;
                let child2 = if index >= 0 {index as usize} else {(children as i32+index) as usize};
                if child2 != child1 {
                    let other = tree.child_map.remove(&Edge(Some(parent), child2)).unwrap();
                    tree.child_map.insert(Edge(Some(parent), child1), other);
                    return MissingEdge(tree, Edge(Some(parent), child2));
                }
            },
            _ => ()
        }
        return MissingEdge(tree, maybe_parent);
    }
}
#[must_use]
pub struct LooseTree<'a,Dat>(&'a mut Tree<Dat>, Id);
impl<'a, Dat> LooseTree<'a,Dat> {
    pub fn export(self) -> Tree<Dat> {
        let LooseTree(tree, root) = self;
        let Node {children, obj, parent} = tree.dat.remove(root.0).unwrap();
        if let Some(maybe_root) = tree.child_map.get(&parent) {
            if *maybe_root == root {
                tree.child_map.remove(&parent);
            }
        }
        let mut new = Tree::new(obj);
        let new_root = new.root();
        for child in 0..children {
            let child_id = tree.child_map.remove(&Edge(Some(root), child)).unwrap();
            let new_child = new.comprehend_other(tree, Edge(Some(new_root), child), child_id);
            new.child_map.insert(Edge(Some(new_root), child), new_child);
        }
        return new;
    }
    pub fn take_top(self) -> Dat {
        let LooseTree(tree, root) = self;
        let Node {children, obj, parent} = tree.dat.remove(root.0).unwrap();
        if let Some(maybe_root) = tree.child_map.get(&parent) {
            if *maybe_root == root {
                tree.child_map.remove(&parent);
            }
        }
        for child in 0..children {
            let child_id = tree.child_map.remove(&Edge(Some(root), child)).unwrap();
            tree.purge_rec(child_id);
        }
        return obj;
    }
    /// Removes the child (along with its parent's acknowledgement of it.  
    /// If this function is called, you MUST change the parent so that 
    /// there is not a blank child slot.
    pub fn purge(self) -> &'a mut Tree<Dat> {
        let LooseTree(tree, root) = self;
        let parent = tree.dat[root.0].parent;
        if let Some(node) = tree.child_map.get(&parent) {
            if *node == root {
                tree.child_map.remove(&parent);
            }
        }
        tree.purge_rec(root);
        return tree;
    }
    /// Places this loose tree in the space of i,
    /// and removes i from 
    pub fn replace(self, i: Id) -> LooseTree<'a,Dat> {
        let LooseTree(tree, loose) = self;
        let MissingAndLoose(tree, missing, other) = tree.cut(i);
        let tree = MissingAndLoose(tree, missing, loose).add_loose();
        return LooseTree(tree, other);
    }
    pub fn append(self, parent: Id) -> MissingAndLoose<'a,Dat> {
        let LooseTree(tree, loose) = self;
        let MissingEdge(tree, edge) = tree.append(parent);
        return MissingAndLoose(tree, edge, loose);
    }
    pub fn insert(self, parent: Id, pos: ChildI) -> MissingAndLoose<'a,Dat> {
        let LooseTree(tree, loose) = self;
        let MissingEdge(tree, edge) = tree.insert(parent, pos);
        return MissingAndLoose(tree, edge, loose);
    }
}
#[must_use]
pub struct MissingAndLoose<'a, Dat>(&'a mut Tree<Dat>, Edge, Id);
impl<'a, Dat> MissingAndLoose<'a, Dat> {
    pub fn collapse(self) -> LooseTree<'a,Dat> {
        let MissingAndLoose(tree, edge, root) = self;
        let tree = MissingEdge(tree,edge).collapse();
        return LooseTree(tree, root);
    }
    pub fn swap(self, i: Id) -> MissingAndLoose<'a,Dat> {
        let MissingAndLoose(tree, edge, root) = self;
        let MissingEdge(tree,next) = MissingEdge(tree,edge).swap(i);
        return MissingAndLoose(tree, next, root);
    }
    pub fn reorder(self, index: i32) -> MissingAndLoose<'a,Dat> {
        let MissingAndLoose(tree, edge, root) = self;
        let MissingEdge(tree,next) = MissingEdge(tree,edge).reorder(index);
        return MissingAndLoose(tree, next, root);
    }
    pub fn add_node(self, val: Dat) -> LooseTree<'a,Dat> {
        let MissingAndLoose(tree, edge, root) = self;
        let _id = MissingEdge(tree,edge).add_node(val);
        return LooseTree(tree, root);
    }
    pub fn add_exttree(self, val: ExtTree<Dat>) -> LooseTree<'a,Dat> {
        let MissingAndLoose(tree, edge, root) = self;
        let _ = MissingEdge(tree,edge).add_exttree(val);
        return LooseTree(tree,root);
    }
    pub fn add_rectree(self, val: (&'a mut Vec<Id>, RecTree<Dat>)) -> LooseTree<'a,Dat> {
        let MissingAndLoose(tree, edge, root) = self;
        let _ = MissingEdge(tree,edge).add_rectree(val);
        return LooseTree(tree,root);
    }
    pub fn add_tree(self, val: Tree<Dat>) -> LooseTree<'a,Dat> {
        let MissingAndLoose(tree, edge, root) = self;
        let _ = MissingEdge(tree,edge).add_tree(val);
        return LooseTree(tree,root);
    }
    pub fn add_loose(self) -> &'a mut Tree<Dat> {
        let MissingAndLoose(tree, edge, root) = self;
        tree.child_map.insert(edge, root);
        tree.dat[root.0].parent = edge;
        return tree;
    }
    /// i CANNOT be an ancestor of the missing edge
    pub fn replace(self, i: Id) -> MissingAndLoose<'a,Dat> {
        let MissingAndLoose(tree, edge, root) = self;
        let LooseTree(tree, new_loose) = LooseTree(tree,root).replace(i);
        return MissingAndLoose(tree, edge, new_loose);
    }
    pub fn replace_ancestor(self, i: Id) -> LooseWithMissing<'a,Dat> {
        let MissingAndLoose(tree, edge, root) = self;
        let LooseTree(tree, new_loose) = LooseTree(tree,root).replace(i);
        return LooseWithMissing(tree, new_loose, edge);
    }
    /// i CANNOT be an ancestor of the missing edge
    pub fn purge(self) -> MissingEdge<'a,Dat> {
        let MissingAndLoose(tree, edge, root) = self;
        let tree = LooseTree(tree,root).purge();
        return MissingEdge(tree, edge);
    }
}
#[must_use]
pub struct LooseWithMissing<'a, Dat>(&'a mut Tree<Dat>, Id, Edge);
impl<'a, Dat> LooseWithMissing<'a, Dat> {
    pub fn purge(self) -> &'a mut Tree<Dat> {
        let LooseWithMissing(tree, loose, _missing) = self;
        return LooseTree(tree,loose).purge();
    }
    pub fn replace(self, i: Id) -> MissingAndLoose<'a,Dat> {
        let LooseWithMissing(tree, loose, missing) = self;
        let LooseTree(tree,new_loose) = LooseTree(tree,loose).replace(i);
        return MissingAndLoose(tree,missing,new_loose);
    }
    pub fn collapse(self) -> LooseTree<'a,Dat> {
        let LooseWithMissing(tree, loose, missing) = self;
        let tree = MissingEdge(tree,missing).collapse();
        return LooseTree(tree,loose);
    }
    pub fn reorder(self, index: i32) -> LooseWithMissing<'a,Dat> {
        let LooseWithMissing(tree, loose, missing) = self;
        let MissingEdge(tree,new_missing) = MissingEdge(tree,missing).reorder(index);
        return LooseWithMissing(tree,loose,new_missing);
    }
    pub fn append(self, parent: Id) -> MissingAndLooseWithMissing<'a,Dat> {
        let LooseWithMissing(tree, loose, missing) = self;
        let MissingEdge(tree, missing2) = tree.append(parent);
        return MissingAndLooseWithMissing(tree,missing2,loose,missing);
    }
    pub fn insert(self, parent: Id, pos: ChildI) -> MissingAndLooseWithMissing<'a,Dat> {
        let LooseWithMissing(tree, loose, missing) = self;
        let MissingEdge(tree, missing2) = tree.insert(parent,pos);
        return MissingAndLooseWithMissing(tree,missing2,loose,missing);
    }
    pub fn add_node(self, val: Dat) -> LooseTree<'a,Dat> {
        let LooseWithMissing(tree, loose, missing) = self;
        MissingEdge(tree,missing).add_node(val);
        return LooseTree(tree, loose);
    }
    pub fn add_exttree(self, val: ExtTree<Dat>) -> LooseTree<'a,Dat> {
        let LooseWithMissing(tree, loose, missing) = self;
        let _ = MissingEdge(tree,missing).add_exttree(val);
        return LooseTree(tree,loose);
    }
    pub fn add_rectree(self, val: (&'a mut Vec<Id>, RecTree<Dat>)) -> LooseTree<'a,Dat> {
        let LooseWithMissing(tree, loose, missing) = self;
        let _ = MissingEdge(tree,missing).add_rectree(val);
        return LooseTree(tree,loose);
    }
    pub fn add_tree(self, val: Tree<Dat>) -> LooseTree<'a,Dat> {
        let LooseWithMissing(tree, loose, missing) = self;
        let _ = MissingEdge(tree,missing).add_tree(val);
        return LooseTree(tree,loose);
    }
}
#[must_use]
pub struct MissingAndLooseWithMissing<'a, Dat>(&'a mut Tree<Dat>, Edge, Id, Edge);
impl<'a, Dat> MissingAndLooseWithMissing<'a, Dat> {
    pub fn purge(self) -> MissingEdge<'a,Dat> {
        let MissingAndLooseWithMissing(tree, missing, loose, _in_loose) = self;
        let tree = LooseTree(tree,loose).purge();
        return MissingEdge(tree,missing);
    }
    pub fn replace_ancestor(self, i: Id) -> MissingAndLooseWithMissing<'a,Dat> {
        let MissingAndLooseWithMissing(tree, missing, loose, in_loose) = self;
        let LooseTree(tree,new_loose) = LooseTree(tree,loose).replace(i);
        return MissingAndLooseWithMissing(tree,in_loose,new_loose,missing);
    }
    pub fn swap(self, i: Id) -> MissingAndLooseWithMissing<'a,Dat> {
        let MissingAndLooseWithMissing(tree, missing, loose, in_loose) = self;
        let MissingEdge(tree,new_edge) = MissingEdge(tree,missing).swap(i);
        return MissingAndLooseWithMissing(tree,new_edge,loose,in_loose);
    }
    pub fn collapse(self) -> LooseWithMissing<'a,Dat> {
        let MissingAndLooseWithMissing(tree, missing, loose, in_loose) = self;
        let tree = MissingEdge(tree,missing).collapse();
        return LooseWithMissing(tree,loose,in_loose);
    }
    pub fn reorder(self, index: i32) -> MissingAndLooseWithMissing<'a,Dat> {
        let MissingAndLooseWithMissing(tree, missing, loose, in_loose) = self;
        let MissingEdge(tree,new_edge) = MissingEdge(tree,missing).reorder(index);
        return MissingAndLooseWithMissing(tree,new_edge,loose,in_loose);
    }
    pub fn add_node(self, val: Dat) -> LooseWithMissing<'a,Dat> {
        let MissingAndLooseWithMissing(tree, missing, loose, in_loose) = self;
        MissingEdge(tree,missing).add_node(val);
        return LooseWithMissing(tree, loose, in_loose);
    }
    pub fn add_exttree(self, val: ExtTree<Dat>) -> LooseWithMissing<'a,Dat> {
        let MissingAndLooseWithMissing(tree, missing, loose, in_loose) = self;
        let _ = MissingEdge(tree,missing).add_exttree(val);
        return LooseWithMissing(tree,loose,in_loose);
    }
    pub fn add_rectree(self, val: (&'a mut Vec<Id>, RecTree<Dat>)) -> LooseWithMissing<'a,Dat> {
        let MissingAndLooseWithMissing(tree, missing, loose, in_loose) = self;
        let _ = MissingEdge(tree,missing).add_rectree(val);
        return LooseWithMissing(tree,loose,in_loose);
    }
    pub fn add_tree(self, val: Tree<Dat>) -> LooseWithMissing<'a,Dat> {
        let MissingAndLooseWithMissing(tree, missing, loose, in_loose) = self;
        let _ = MissingEdge(tree,missing).add_tree(val);
        return LooseWithMissing(tree,loose,in_loose);
    }
    pub fn add_loose(self) -> MissingEdge<'a,Dat> {
        let MissingAndLooseWithMissing(tree, missing, loose, in_loose) = self;
        let tree = MissingAndLoose(tree,missing,loose).add_loose();
        return MissingEdge(tree,in_loose);
    }
}

#[derive(Clone,Copy,Default,Debug,Hash,PartialEq,Eq)]
#[derive(serde::Serialize,serde::Deserialize)]
pub struct Id(usize);
type ChildI = usize;
#[derive(Clone,Copy,Debug,Hash,PartialEq,Eq)]
#[derive(serde::Serialize,serde::Deserialize)]
pub struct Edge(Option<Id>,ChildI);

#[derive(serde::Serialize,serde::Deserialize)]
struct Node<Dat> {
    parent: Edge,
    obj: Dat,
    children: usize
}
#[derive(Debug,PartialEq,Eq)]
pub enum TreeProblem {
    /// (Parent, child_num), child with wrong parent
    IncorrectParent(Edge,Id),
    UnreachableNodes(usize),
    NonexistentChild(Id),
    NonexistentLink(Edge),
    TooManyEdges(usize)
}

/// PAY ATTENTION TO THE UNUSED WARNINGS! I CAN'T GET RUST
/// TO MAKE THEM THROW AN ERROR, BUT THEY ARE ALL ERRORS!
/// NOT CALLING A FUNCTION ON THE RESULTING OBJECT BREAKS THE TREE!
#[derive(serde::Serialize,serde::Deserialize)]
pub struct Tree<Dat> {
    dat: RemoveVec<Node<Dat>>,
    child_map: HashMap<Edge, Id>,
} impl<Dat> Tree<Dat> {
    pub fn new(obj: Dat) -> Self {
        let mut dat = RemoveVec::new();
        let root = dat.push(Node {obj, parent: Edge(None,0), children: 0});
        let mut child_map = HashMap::new();
        child_map.insert(Edge(None,0), Id(root));
        Self {
            dat,
            child_map
        }
    }
    
    /************** INTERACTIVITY FUNCTIONS ***************************/
    
    /// Allows the mutable modification of a parent, while also having immutable access to its own
    /// children.  
    /// You're welcome.
    pub fn read_children<'a>(&'a mut self, pid: Id) -> (&'a mut Dat, Vec<&'a Dat>) {
        let num_children = self.dat[pid.0].children;
        let mut v = Vec::with_capacity(num_children);
        let parent;
        unsafe {
            let vec = self.dat.interior();
            let len = vec.len();
            let ptr = vec.as_mut_ptr();
            let ploc = RemoveVec::<()>::chop(pid.0).1;
            assert!(ploc<len);
            parent = &mut (*ptr.add(ploc)).as_mut().unwrap().obj;
            
            for child in 0..num_children {
                let child_id = self.child_map[&Edge(Some(pid),child)];
                let cloc = RemoveVec::<()>::chop(child_id.0).1;
                assert!(cloc<len&&ploc != cloc);
                v.push(&(*ptr.add(cloc)).as_ref().unwrap().obj);
            }
            
        }
        
        return (parent, v);
    }

    pub fn write_children<'a>(&'a mut self, pid: Id) -> (&'a mut Dat, Vec<&'a mut Dat>) {
        let num_children = self.dat[pid.0].children;
        let mut v = Vec::with_capacity(num_children);
        let parent;
        unsafe {
            let vec = self.dat.interior();
            let len = vec.len();
            let ptr = vec.as_mut_ptr();
            let ploc = RemoveVec::<()>::chop(pid.0).1;
            assert!(ploc<len);
            parent = &mut (*ptr.add(ploc)).as_mut().unwrap().obj;
            
            for child in 0..num_children {
                let child_id = self.child_map[&Edge(Some(pid),child)];
                let cloc = RemoveVec::<()>::chop(child_id.0).1;
                assert!(cloc<len&&ploc != cloc);
                v.push(&mut (*ptr.add(cloc)).as_mut().unwrap().obj);
            }
            
        }
        
        return (parent, v);
    }
    
    
    /*************   RECURSIVE HELPER FUNCTIONS    ******************/
    
    fn comprehend(&mut self, parent: Id, tree: ExtTree<Dat>) {
        let ExtTree(here, children) = tree;
        let id = self.append(parent).add_node(here);
        for child in children {
            self.comprehend(id, child);
        }
    }
    fn comprehend_record(&mut self, parent: Id, tree: ExtTree<(bool,Dat)>, saved: &mut Vec<Id>) {
        let ExtTree((keep, here), children) = tree;
        let id = self.append(parent).add_node(here);
        if keep {saved.push(id)}
        for child in children {
            self.comprehend_record(id, child, saved);
        }
    }
    fn comprehend_other(&mut self, other: &mut Tree<Dat>, making: Edge, from: Id) -> Id {
        let Node {children, obj, ..} = other.dat.remove(from.0).unwrap();
        let new = Id(self.dat.push(Node {obj, parent: making, children}));
        for child in 0..children {
            let child_id = other.child_map.remove(&Edge(Some(from), child)).unwrap();
            let new_child = self.comprehend_other(other, Edge(Some(new), child), child_id);
            self.child_map.insert(Edge(Some(new), child), new_child);
        }
        return new;
    }
    fn purge_rec(&mut self, i: Id) {
        let len = self.dat[i.0].children;
        for child in 0..len {
            if let Some(child_id) = self.child_map.remove(&Edge(Some(i),child)) {
                self.purge_rec(child_id);
            }
        }
        self.dat.remove(i.0);
    }
    
    /*************   MODIFICATION FUNCTIONS   ******************/
    
    pub fn append<'a>(&'a mut self, parent: Id) -> MissingEdge<'a,Dat> {
        let child = self.dat[parent.0].children;
        self.dat[parent.0].children += 1;
        return MissingEdge(self, Edge(Some(parent),child));
    }
    pub fn insert<'a>(&'a mut self, parent: Id, pos: ChildI) -> MissingEdge<'a, Dat> {
        let children = self.dat[parent.0].children;
        let position = if pos <= children {pos} else {children};
        self.dat[parent.0].children += 1;
        for child in (position..children).rev() {
            let old_edge = Edge(Some(parent), child);
            let new_edge = Edge(Some(parent), child+1);
            let child_id = self.child_map[&old_edge];
            self.dat[child_id.0].parent = new_edge;
            self.child_map.insert(new_edge, child_id);
        }
        return MissingEdge(self, Edge(Some(parent),pos));
    }
    pub fn cut<'a>(&'a mut self, i: Id) -> MissingAndLoose<'a,Dat> {
        let parent = self.dat[i.0].parent;
        self.dat[i.0].parent = Edge(None, 1);
        self.child_map.remove(&parent);
        return MissingAndLoose(self, parent, i);
    }
    pub fn swap(&mut self, i1: Id, i2: Id) -> &mut Self {
        return self.cut(i1).swap(i2).add_loose();
    }
    pub fn swap_ancestor<'a>(&'a mut self, ancestor: Id, descendant: Id) -> LooseWithMissing<'a,Dat> {
        return self.cut(descendant).replace_ancestor(ancestor);
    }
    
    /*************   INFORMATIONAL FUNFCTIONS ******************/
    pub fn equals(&self, other: &Tree<Dat>) -> bool where Dat: PartialEq+Eq {
        if self[self.root()] != other[other.root()] {
            return false;
        }
        let (mut iter1,mut iter2) = (self.df_forward(self.root(), 0), other.df_forward(other.root(), 0));
        
        loop {
            match (iter1.next(self), iter2.next(other)) {
                (Some(mut ext1), Some(mut ext2)) => {
                    let (id1, above1) = ext1.receive();
                    let (id2, above2) = ext2.receive();
                    let (depth1, depth2) = (*above1+1, *above2+1);
                    if above1 != above2 || self[id1] != other[id2] {
                        return false;
                    }
                    iter1 = ext1.provide(depth1);
                    iter2 = ext2.provide(depth2);
                },
                (None,None) => return true,
                _ => return false
            }
        }
    }
    /*
    fn in_loose(&self, i: Id) -> bool {
        let mut parent = self.dat[i.0].parent;
        while let Edge(Some(next_node), _child) = parent {
            parent = self.dat[next_node.0].parent;
            if let Edge(None,0) = parent {
                return false;
            }
            if let Some(id) = self.child_map.get(&parent) {
                if *id != next_node {
                    return true;
                }
            }
        }
        true
    }
    */
    /// Returns the path to this node, starting from the root.
    pub fn from_root(&self, i: Id) -> Vec<Id> {
        let mut path = vec![i];
        let mut parent = self.dat[i.0].parent;
        while let Edge(Some(next_node), _child) = parent {
            path.push(next_node);
            parent = self.dat[next_node.0].parent;
        }
        path.reverse(); path
    }
    pub fn ancestor_of(&self, i1: Id, i2: Id) -> bool {
        let mut parent = self.dat[i2.0].parent;
        while let Edge(Some(next_node), _child) = parent {
            if next_node == i1 {
                return true;
            }
            parent = self.dat[next_node.0].parent;
        }
        false
    }
    pub fn len(&self) -> usize {
        return self.dat.len();
    }
    /// Returns the common ancestor of i1 and i2.
    pub fn common_ancestor(&self, i1: Id, i2: Id) -> Id {
        let (path1, path2) = (self.from_root(i1), self.from_root(i2));
        for index in 1..path1.len().min(path2.len()) {
            if path1[index] != path2[index] {
                return path1[index-1];
            }
        }
        return path1[0];
    }
    pub fn descendant(&self, ansc: Id, path: &[ChildI]) -> Option<Id> {
        let mut current = ansc;
        for next in path {
            current = *self.child_map.get(&Edge(Some(current),*next))?;
        }
        return Some(current);
    }
    pub fn child(&self, parent: Id, index: i32) -> Id {
        let children = self.dat[parent.0].children;
        let child = if index >= 0 {index as usize} else {(children as i32+index) as usize};
        return self.child_map[&Edge(Some(parent),child)];
    }
    pub fn root(&self) -> Id {self.child_map[&Edge(None,0)]}
    pub fn has(&self, i: Id) -> bool {
        return self.dat.has(i.0);
    }
    /// Returns the number of children of a node
    pub fn children(&self, i: Id) -> usize {
        return self.dat[i.0].children;
    }
    /// Returns the parent of this node.  
    /// If this is the root, or the node does not exist,
    /// returns the same id.
    pub fn parent(&self, i: Id) -> Option<Id> {
        if let Some(p) = self.dat[i.0].parent.0 {Some(p)} else {None}
    }
    pub fn child_num(&self, parent: Id, child: Id) -> Option<usize> {
        if let Edge(Some(actual_parent),child) = self.dat[child.0].parent {
            if parent == actual_parent {return Some(child);}
        }
        None
    }
    /// Returns the location of this node in the tree as Option<(parent, # of this child)>
    pub fn location(&self, i: Id) -> Option<(Id,ChildI)> {
        match self.dat[i.0].parent {
            Edge(None,_) => None,
            Edge(Some(p),x) => Some((p,x))
        }
    }
    pub fn depth(&self, mut i: Id) -> usize {
        let mut count = 0;
        loop {
            if let Some(p) = self.dat[i.0].parent.0 {
                count += 1;
                i = p;
            } else {return count;}
        }
    }
    
    pub fn df_forward<T>(&self, start_id: Id, start: T) -> DFForwardStoreIter<T> {
        return DFForwardStoreIter::new(start_id, start);
    }
    pub fn df_reverse<T>(&self, start_id: Id) -> DFReverseStoreIter<T> {
        return DFReverseStoreIter::new(start_id);
    }
    
    fn sanity_check(&self, parent: Edge, i: Id) -> Result<(usize, usize), TreeProblem> {
        if !self.dat.has(i.0) {
            return Err(TreeProblem::NonexistentChild(i));
        }
        if self.dat[i.0].parent != parent {
            return Err(TreeProblem::IncorrectParent(parent,i));
        }
        let children = self.dat[i.0].children;
        let mut descendants = 1;
        let mut edge_count = 0;
        for child in 0..children {
            edge_count += 1;
            if let Some(child_id) = self.child_map.get(&Edge(Some(i),child)) {
                let (new_nodes, new_edges) = self.sanity_check(Edge(Some(i),child), *child_id)?;
                descendants += new_nodes;
                edge_count += new_edges;
            } else {return Err(TreeProblem::NonexistentLink(Edge(Some(i),child)))}
        }
        Ok((descendants, edge_count))
    }
    /// This is a function to check if the tree is still 'sane',
    /// that is, none of the fundamental assumptions of being a tree
    /// have been violated. No accessible function should be able to break the tree's sanity,
    /// but if problems occur this can check.
    pub fn sane(&self) -> Result<(), TreeProblem> {
        let root = self.root();
        let (reachable_nodes, used_edges) = self.sanity_check(Edge(None,0), root)?;
        if reachable_nodes != self.dat.len() {
            return Err(TreeProblem::UnreachableNodes(self.dat.len()-reachable_nodes));
        }
        if used_edges+1 != self.child_map.len() {
            return Err(TreeProblem::TooManyEdges(self.child_map.len()-used_edges));
        }
        Ok(())
    }
}
impl<Dat> std::ops::Index<Id> for Tree<Dat> {
    type Output = Dat;
    fn index(&self, i: Id) -> &Self::Output {
        return &self.dat[i.0].obj;
    }
}
impl<Dat> std::ops::IndexMut<Id> for Tree<Dat> {
    fn index_mut(&mut self, i: Id) -> &mut Self::Output {
        return &mut self.dat[i.0].obj;
    }
}

/// Notably, this iterator never provides the root node.  
/// The first node must be manually accessed and calculated.
pub struct DFForwardStoreIter<T>(Vec<(Id, usize, T)>);
impl<T> DFForwardStoreIter<T> {
    pub fn new(start: Id, x: T) -> Self {
        Self(vec![(start, 0, x)])
    }
    pub fn next<Dat>(mut self, tree: &Tree<Dat>) -> Option<DFForwardStoreExtract<T>> {
        if self.0.len() == 0 {
            return None;
        }
        let (mut id, mut num_child, _) = self.0[self.0.len()-1];
        while num_child >= tree.children(id) {
            self.0.pop();
            if self.0.len() == 0 {
                return None;
            }
            (id, num_child, _) = self.0[self.0.len()-1];
        }
        let last = self.0.len()-1;
        let (parent, num_child, _) = self.0[last];
        self.0[last].1 += 1;
        let child = tree.child(parent, num_child as i32);
        return Some(DFForwardStoreExtract {iter: self.0, here: (child,0)});
    }
    pub fn here(&self) -> (Id, &T) {
        let len = self.0.len();
        let (a,_,c) = &self.0[len-1];
        return (*a,c);
    }
}
pub struct DFForwardStoreExtract<T> {
    iter: Vec<(Id, usize, T)>,
    here: (Id,usize)
} impl<T> DFForwardStoreExtract<T> {
    pub fn provide(mut self, val: T) -> DFForwardStoreIter<T> {
        self.iter.push((self.here.0,self.here.1, val));
        DFForwardStoreIter(self.iter)
    }
    pub fn receive(&mut self) -> (Id, &mut T) {
        let len = self.iter.len();
        return (self.here.0,&mut self.iter[len-1].2);
    }
}

pub enum ItrResult<A,B> {Continue(A), Stop(B)}

pub struct ReverseExtract<T> {
    pub descendant: Option<T>,
    pub antecedent: Option<T>,
    pub id: Id
}
pub enum DFReverseStoreIter<T> {
    Continue(Vec<(Id, usize, Option<T>)>),
    Stop(T)
}
impl<T> DFReverseStoreIter<T> {
    pub fn new(start: Id) -> Self {
        Self::Continue(vec![(start, 0, None)])
    }
    pub fn next<Dat>(mut self, tree: &Tree<Dat>) -> ItrResult<(ReverseExtract<T>, DFReverseStoreIter2<T>), T> {
        match self {
            Self::Continue(mut iter) => {
                let (mut id, mut num_child, _) = iter[iter.len()-1];
                while num_child < tree.children(id) {
                    iter.push((tree.child(id, num_child as i32), 0, None));
                    (id, num_child, _) = iter[iter.len()-1];
                }
                let (id, num_child, descendant) = iter.pop().unwrap();
                if iter.len() != 0 {
                    let last = iter.len()-1;
                    let antecedent = iter[last].2.take();
                    iter[last].1 += 1;
                    return ItrResult::Continue((ReverseExtract {descendant, antecedent, id}, DFReverseStoreIter2(iter)));
                } else {
                    return ItrResult::Continue((ReverseExtract {descendant, antecedent: None, id}, DFReverseStoreIter2(iter)));
                }
            }
            Self::Stop(n) => ItrResult::Stop(n)
        }
    }
}
pub struct DFReverseStoreIter2<T>(Vec<(Id, usize, Option<T>)>);
impl<T> DFReverseStoreIter2<T> {
    pub fn provide(mut self, val: T) -> DFReverseStoreIter<T> {
        let len = self.0.len();
        if len > 0 {
            self.0[len-1].2 = Some(val);
            DFReverseStoreIter::Continue(self.0)
        } else {
            DFReverseStoreIter::Stop(val)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    fn tree_complex() -> (Vec<Id>, Tree::<String>) {
        let mut vars = Vec::new();
        let tree = Tree::<String>::from((&mut vars,
            ExtTree((true, "root".to_string()), vec![
                ExtTree((false, "0".to_string()), vec![]),
                ExtTree((false, "1".to_string()), vec![
                    ExtTree((true, "1,0".to_string()), vec![
                        ExtTree((false, "1,0,0".to_string()), vec![]),
                        ExtTree((false, "1,0,1".to_string()), vec![]),
                        ExtTree((false, "1,0,2".to_string()), vec![]),
                    ])
                ]),
                ExtTree((true, "2".to_string()), vec![
                    ExtTree((false, "2,0".to_string()), vec![
                        ExtTree((true, "2,0,0".to_string()), vec![]),
                        ExtTree((false, "2,0,1".to_string()), vec![])
                    ]),
                    ExtTree((false, "2,1".to_string()), vec![
                        ExtTree((false, "2,1,0".to_string()), vec![]),
                        ExtTree((false, "2,1,1".to_string()), vec![]),
                        ExtTree((false, "2,1,2".to_string()), vec![]),
                    ])
                ]),
                ExtTree((false, "3".to_string()), vec![
                    ExtTree((false, "3,0".to_string()), vec![]),
                    ExtTree((false, "3,1".to_string()), vec![
                        ExtTree((false, "3,1,0".to_string()), vec![
                            ExtTree((false, "3,1,0,0".to_string()), vec![
                                ExtTree((false, "3,1,0,0,0".to_string()), vec![])
                            ])
                        ])
                    ]),
                    ExtTree((false, "3,2".to_string()), vec![
                        ExtTree((false, "3,2,0".to_string()), vec![
                            ExtTree((false, "3,2,0,0".to_string()), vec![]),
                            ExtTree((false, "3,2,0,1".to_string()), vec![])
                        ]),
                        ExtTree((false, "3,2,1".to_string()), vec![
                            ExtTree((false, "3,2,1,0".to_string()), vec![]),
                            ExtTree((false, "3,2,1,1".to_string()), vec![])
                        ]),
                    ]),
                    ExtTree((true, "3,3".to_string()), vec![
                        ExtTree((false, "3,3,0".to_string()), vec![]),
                        ExtTree((false, "3,3,1".to_string()), vec![]),
                        ExtTree((false, "3,3,2".to_string()), vec![]),
                        ExtTree((false, "3,3,3".to_string()), vec![]),
                        ExtTree((false, "3,3,4".to_string()), vec![]),
                        ExtTree((true, "3,3,5".to_string()), vec![]),
                        ExtTree((false, "3,3,6".to_string()), vec![]),
                        ExtTree((false, "3,3,7".to_string()), vec![]),
                        ExtTree((false, "3,3,8".to_string()), vec![]),
                        ExtTree((false, "3,3,9".to_string()), vec![]),
                        ExtTree((false, "3,3,10".to_string()), vec![]),
                    ]),
                ]),
            ])
        ));
        return (vars, tree);
    }
    fn tree_medium() -> Tree<String> {
        Tree::from(
        ExtTree("root".to_string(), vec![
            ExtTree("0".to_string(), vec![]),
            ExtTree("1".to_string(), vec![
                ExtTree("1,0".to_string(), vec![
                    ExtTree("1,0,0".to_string(), vec![]),
                    ExtTree("1,0,1".to_string(), vec![]),
                    ExtTree("1,0,2".to_string(), vec![]),
                ])
            ]),
            ExtTree("2".to_string(), vec![
                ExtTree("2,0".to_string(), vec![
                    ExtTree("2,0,0".to_string(), vec![]),
                    ExtTree("2,0,1".to_string(), vec![])
                ]),
                ExtTree("2,1".to_string(), vec![
                    ExtTree("2,1,0".to_string(), vec![]),
                    ExtTree("2,1,1".to_string(), vec![]),
                    ExtTree("2,1,2".to_string(), vec![]),
                ])
            ]),
        ]))
    }
    
    #[test]
    fn no_ops() {
        #![allow(unused_variables)]
        let (vars, mut tree1) = tree_complex();
        let (_,tree2) = tree_complex();
        let [root, n_1_0, n_2, n_2_0_0, n_3_3, n_3_3_5] = vars[..] else {panic!()};
        tree1.insert(n_3_3, 4).collapse();
        check(&tree1, &tree2);
        tree1.insert(n_3_3, 0).collapse();
        check(&tree1, &tree2);
        tree1.insert(n_3_3, 10).collapse();
        check(&tree1, &tree2);
        tree1.append(n_3_3).collapse();
        check(&tree1, &tree2);
        tree1.insert(n_3_3_5,4).collapse();
        check(&tree1, &tree2);
        tree1.append(n_3_3_5).collapse();
        check(&tree1, &tree2);
        tree1.cut(n_3_3).add_loose();
        check(&tree1, &tree2);
        tree1.append(n_2).reorder(0).reorder(2).collapse();
        check(&tree1, &tree2);
        tree1.swap(n_2,n_1_0).swap(n_1_0,n_2);
        check(&tree1, &tree2);
    }
    
    fn check<Dat: Eq>(checking: &Tree<Dat>, what_it_should_be: &Tree<Dat>) {
        assert_eq!(checking.sane(), Ok(()));
        assert!(checking.equals(what_it_should_be));
    }

    #[test]
    fn one_node() {
        let tree = Tree::new("hello".to_string());
        assert_eq!(tree.sane(), Ok(()));
        assert_eq!(tree.len(), 1);
        let root = tree.root();
        assert_eq!(tree[root], "hello".to_string());
        assert_eq!(tree.children(root), 0);
    }
    
    #[test]
    fn transfer() {
        
    }
    
    #[test]
    fn constructors() {
        let mut vars = Vec::new();
        let tree = Tree::new("hello".to_string());
        assert_eq!(
            Tree::from(ExtTree("hello".to_string(), vec![])
            ).sane(), Ok(())
        );
        assert!(tree.equals(
            &Tree::from(ExtTree("hello".to_string(), vec![])
            )
        ));
        assert_eq!(
            Tree::from((&mut vars, ExtTree((false, "hello".to_string()), vec![])
            )).sane(), Ok(())
        );
        assert!(tree.equals(
            &Tree::from((&mut vars, ExtTree((false, "hello".to_string()), vec![])
            ))
        ));
        
        let mut tree1 = Tree::new("root".to_string());
        let root = tree1.root();
        let _ = tree1.append(root).add_node("0".to_string());
        let one = tree1.append(root).add_node("1".to_string());
        let two = tree1.append(root).add_node("2".to_string());
        let one_zero = tree1.append(one).add_node("1,0".to_string());
        let two_zero = tree1.append(two).add_node("2,0".to_string());
        let two_one = tree1.append(two).add_node("2,1".to_string());
        let _ = tree1.append(one_zero).add_node("1,0,0".to_string());
        let _ = tree1.append(one_zero).add_node("1,0,1".to_string());
        let _ = tree1.append(one_zero).add_node("1,0,2".to_string());
        let _ = tree1.append(two_zero).add_node("2,0,0".to_string());
        let _ = tree1.append(two_zero).add_node("2,0,1".to_string());
        let _ = tree1.append(two_one).add_node("2,1,0".to_string());
        let _ = tree1.append(two_one).add_node("2,1,1".to_string());
        let _ = tree1.append(two_one).add_node("2,1,2".to_string());
    
        let tree2 = Tree::<String>::from((&mut vars,
            ExtTree((true, "root".to_string()), vec![
                ExtTree((false, "0".to_string()), vec![]),
                ExtTree((false, "1".to_string()), vec![
                    ExtTree((true, "1,0".to_string()), vec![
                        ExtTree((false, "1,0,0".to_string()), vec![]),
                        ExtTree((false, "1,0,1".to_string()), vec![]),
                        ExtTree((false, "1,0,2".to_string()), vec![]),
                    ])
                ]),
                ExtTree((true, "2".to_string()), vec![
                    ExtTree((false, "2,0".to_string()), vec![
                        ExtTree((true, "2,0,0".to_string()), vec![]),
                        ExtTree((false, "2,0,1".to_string()), vec![])
                    ]),
                    ExtTree((false, "2,1".to_string()), vec![
                        ExtTree((false, "2,1,0".to_string()), vec![]),
                        ExtTree((false, "2,1,1".to_string()), vec![]),
                        ExtTree((false, "2,1,2".to_string()), vec![]),
                    ])
                ]),
            ])
        ));
        assert_eq!(
            tree1.sane(), Ok(())
        );
        assert_eq!(
            tree2.sane(), Ok(())
        );
        assert!(tree1.equals(&tree2));
        assert!(tree2.equals(&tree1));
    }
    
    #[test]
    fn depth_forward() {
        let tree = tree_medium();
        let mut df_store = tree.df_forward(tree.root(), 0);
        let mut v = Vec::new();
        while let Some(mut ext) = df_store.next(&tree) {
            let (id, above) = ext.receive();
            let here = *above+1;
            v.push((here,tree[id].clone()));
            df_store = ext.provide(here);
            let (id, here) = df_store.here();
        }
        assert!(
            v ==
            vec![
                (1,"0".to_string()),
                (1,"1".to_string()),
                (2,"1,0".to_string()),
                (3,"1,0,0".to_string()),
                (3,"1,0,1".to_string()),
                (3,"1,0,2".to_string()),
                (1,"2".to_string()),
                (2,"2,0".to_string()),
                (3,"2,0,0".to_string()),
                (3,"2,0,1".to_string()),
                (2,"2,1".to_string()),
                (3,"2,1,0".to_string()),
                (3,"2,1,1".to_string()),
                (3,"2,1,2".to_string()),
            ]
        );
    }
    
    #[test]
    fn depth_reverse() {
        let (_,tree) = tree_complex();
        let mut df_store = tree.df_reverse(tree.root());
        let result = loop {
            break match df_store.next(&tree) {
                ItrResult::Continue((ReverseExtract {antecedent, descendant, id}, itr)) => {
                    match (antecedent, descendant) {
                        (None,       None       ) => df_store = itr.provide(vec![tree[id].clone()]),
                        (Some(mut a),Some(mut d)) => df_store = {a.append(&mut d); a.push(tree[id].clone()); itr.provide(a)},
                        (Some(mut a),None       ) => df_store = {a.push(tree[id].clone()); itr.provide(a)},
                        (None,       Some(mut d)) => df_store = {d.push(tree[id].clone()); itr.provide(d)},
                    }
                    continue;
                }
                ItrResult::Stop(t) => t
            }
        };
        println!("{:?}", result);
        assert!(
            result ==
            ["0", "1,0,0", "1,0,1", "1,0,2", "1,0", "1", "2,0,0", "2,0,1", "2,0", "2,1,0", "2,1,1", "2,1,2", "2,1", "2",
             "3,0", "3,1,0,0,0", "3,1,0,0", "3,1,0", "3,1", "3,2,0,0", "3,2,0,1", "3,2,0", "3,2,1,0", "3,2,1,1", "3,2,1", "3,2",
             "3,3,0", "3,3,1", "3,3,2", "3,3,3", "3,3,4", "3,3,5", "3,3,6", "3,3,7", "3,3,8", "3,3,9", "3,3,10", "3,3", "3",
             "root"].iter().map(|s| s.to_string()).collect::<Vec<_>>()
        );
    }
    
    #[test]
    fn separate_read() {
        let (vars,mut tree) = tree_complex();
        #[allow(unused_variables)]
        let [root, one_zero,two,two_zero_zero,three_three,three_three_five] = vars[..] else {panic!()};
        let mut v = Vec::new();
        {
        let (three_three, children) = tree.read_children(three_three);
        *three_three = "test".to_string();
        for x in children {
            v.push(x.clone());
        }
        }
        assert!(tree[three_three] == "test");
        println!("{:?}", v);
        assert!(v == ["3,3,0", "3,3,1", "3,3,2", "3,3,3", "3,3,4", "3,3,5",
        "3,3,6", "3,3,7", "3,3,8", "3,3,9", "3,3,10"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
    }
}
