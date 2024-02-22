pub use applications::common::App;

impl super::AppL for applications::common::App {
    fn launch(&self) {
        todo!()
    }

    fn scrubber(_config: &super::Config) -> Result<Vec<Self>, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        Result::Ok(applications::get_apps())
    }
}
