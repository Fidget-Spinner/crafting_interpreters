trait PastryVisitor<T> {
    fn visit_beignet(&self);
    fn visit_cruller(&self);
}

trait Pastry<T> {
    fn accept(visitor: impl PastryVisitor<T>) -> T;
}

struct Beignet {}
impl <T> Pastry<T> for Beignet {
    fn accept(visitor: impl PastryVisitor<T>) -> T {
        visitor.visit_beignet()
    }
}

struct Cruller {}
impl <T> Pastry<T> for Cruller {
    fn accept(visitor: impl PastryVisitor<T>) -> T {
        visitor.visit_cruller()
    }
}
