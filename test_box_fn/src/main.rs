fn main() {
    let m1 = solver(0);
    test_solver(&m1);
    let m2 = solver(1);
    test_solver(&m2);
    let m3 = solver(33);
    test_solver(&m3);

    let p = (3,4);
    let i = match p {
        (3, y) if y != 4 => 'a',
        (x, 4) => 'b',
        _ => 'c',
    };

    println!("match {:?} -> {}", p, i);
}

fn test_solver(m: &Model) {
    m.calc(0.0);
    m.calc(0.001);
    m.calc(0.01);
    m.calc(0.1);
    m.calc(1.0);
    m.calc(11.0);
    m.calc(100.0);
}

fn solver(u: i32) -> Model {
    let func_k = move |disp: f64| {
        let g1 = (u / 100) as f64;
        let gu = disp / 0.01;
        let g12 = g1 * g1;
        let gu2 = gu * gu; 
        (g12 * (g12 + gu2).powf(-1.5) - 1.0)
    };
    let mut model = Model::new();
    model.set_k_func(Box::new(func_k));
    model
}

struct Model {
    func: Box<dyn Fn(f64) -> f64>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            func: Box::new(|f| f)
        }
    }
    pub fn set_k_func(&mut self, func: Box<dyn Fn(f64) -> f64>) -> &mut Self {
        self.func = func;
        self
    }
    pub fn calc(&self, disp: f64) -> f64 {
        (self.func)(disp)
    }
}