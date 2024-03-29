#![no_std]
#![feature(custom_test_frameworks, start, lang_items, core_intrinsics, alloc_error_handler)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::runner)]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[alloc_error_handler]
fn alloc_error(_: core::alloc::Layout) -> ! {
    panic!("Alloc error")
}

pub use fancy_test_macros::unit_test;

use ferros::*;
use ferros::cap::*;
use ferros::userland::*;

pub type TestName = arrayvec::ArrayString<[u8; 128]>;

pub enum TestEvent {
    TestStarting(TestName),
    TestPassed(TestName),
    AllTestsComplete,
}

pub struct TestContext<Role: CNodeRole> {
    pub x: u32,
    pub test_event_sender: Sender<TestEvent, Role>,
}

impl RetypeForSetup for TestContext<role::Local> {
    type Output = TestContext<role::Child>;
}

impl TestContext<role::Local> {
    pub fn report_test_starting(&self, name: &str) {
        let ev = TestEvent::TestStarting(TestName::from(name).expect("test name too long"));
        self.test_event_sender.blocking_send(&ev).expect("Could not send test event");
    }

    pub fn report_test_passed(&self, name: &str) {
        let ev = TestEvent::TestPassed(TestName::from(name).expect("test name too long"));
        self.test_event_sender.blocking_send(&ev).expect("Could not send test event");
    }

    pub fn complete(&self) -> ! {
        let ev = TestEvent::AllTestsComplete;
        self.test_event_sender.blocking_send(&ev).expect("Could not send test event");

        unsafe {
            loop {
                selfe_sys::seL4_Yield();
            }
        }
    }
}

pub trait Testable {
    fn name(&self) -> &'static str;
    fn run(&self);
}

pub struct UnitTest {
    pub name: &'static str,
    pub f: fn(),
}

impl Testable for UnitTest {
    fn name(&self) -> &'static str {
        self.name
    }

    fn run(&self) {
        (self.f)()
    }
}

const FILTER: Option<&'static str> = option_env!("FERROS_TEST_FILTER");
static mut TEST_CONTEXT: Option<TestContext<role::Local>> = None;

pub fn set_test_context(context: TestContext<role::Local>) {
    unsafe {TEST_CONTEXT = Some(context) };
}

pub fn runner(tests: &[&dyn Testable]) {
    let filter_text = FILTER.unwrap_or("");
    let context = unsafe {TEST_CONTEXT.take().expect("TEST_CONTEXT was not set") };

    for t in tests {
        if FILTER.is_none() || t.name() == filter_text {
            debug_print!("  - {} ...", t.name());
            context.report_test_starting(t.name());
            t.run();
            debug_println!(" PASS");
            context.report_test_passed(t.name());
        }
    }

    context.complete();
}
