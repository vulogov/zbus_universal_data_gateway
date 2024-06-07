pub trait Indicator<In, Out> {
    fn next(&mut self, input: In) -> Out;

    fn reset(&mut self);
}
