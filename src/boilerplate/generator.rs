use crate::boilerplate::cpp::CppGenerator;
use crate::boilerplate::java::JavaGenerator;

pub trait BoilerPlateGenerator {
    fn new(input : &str) -> Self where Self: Sized;
    fn generate(&self) -> String;
    fn needs_boilerplate(&self) -> bool;
}

pub struct BoilerPlate<T: ?Sized> where T: BoilerPlateGenerator {
    generator : Box<T>
}

impl<T: ?Sized> BoilerPlate<T> where T: BoilerPlateGenerator{
    pub fn new(generator : Box<T>) -> Self {
        Self {
            generator
        }
    }

    pub fn generate(&self) -> String {
        self.generator.generate()
    }

    pub fn needs_boilerplate(&self) -> bool {
        self.generator.needs_boilerplate()
    }
}

pub struct Null;
impl BoilerPlateGenerator for Null {
    fn new(_: &str) -> Self {
        Self {}
    }

    fn generate(&self) -> String {
        panic!("Cannot generate null boilerplate!");
    }

    fn needs_boilerplate(&self) -> bool {
        false
    }
}

pub fn boilerplate_factory(language : &str, code : &str)
    -> BoilerPlate<dyn BoilerPlateGenerator> {
    return match language {
        "c++" | "c" => {
            BoilerPlate::new(Box::new(CppGenerator::new(code)))
        }
        "java" => {
            BoilerPlate::new(Box::new(JavaGenerator::new(code)))
        }
        unknown => {
            warn!("Unknown boilerplate for: {}", unknown);
            BoilerPlate::new(Box::new(Null::new(code)))
        }
    }
}