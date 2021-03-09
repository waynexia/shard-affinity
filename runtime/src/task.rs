use std::cell::UnsafeCell;
use std::future::Future;
use std::mem::{forget, ManuallyDrop};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::{self, Relaxed};
use std::sync::mpsc::Sender;
use std::task::{Context, RawWaker, RawWakerVTable, Waker};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

const WAITING: u8 = 0; // --> POLLING
const POLLING: u8 = 1; // --> WAITING, REPOLL, or COMPLETE
const REPOLL: u8 = 2; // --> POLLING
const COMPLETE: u8 = 3; // No transitions out

/// default ordering
const ORDERING: Ordering = Relaxed;

/// `!Sync`
struct Task {
    task: UnsafeCell<BoxFuture<'static, ()>>,
    queue: Sender<RcTask>,
    status: AtomicU8,
}

unsafe impl Send for Task {}

#[derive(Clone)]
pub struct RcTask(Rc<Task>);

// todo: this send on Rc
unsafe impl Send for RcTask {}

impl RcTask {
    #[inline]
    pub fn new<F>(future: F, queue: Sender<RcTask>) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let future = Rc::new(Task {
            task: UnsafeCell::new(Box::pin(future)),
            queue,
            status: AtomicU8::new(WAITING),
        });
        let future: *const Task = Rc::into_raw(future) as *const Task;
        unsafe { task(future) }
    }

    #[inline]
    pub unsafe fn poll(self) {
        self.0.status.store(POLLING, ORDERING);
        let waker = ManuallyDrop::new(waker(&*self.0));
        let mut cx = Context::from_waker(&waker);
        loop {
            if Pin::new(&mut *self.0.task.get()).poll(&mut cx).is_ready() {
                break self.0.status.store(COMPLETE, ORDERING);
            }
            match self
                .0
                .status
                .compare_exchange(POLLING, WAITING, ORDERING, ORDERING)
            {
                Ok(_) => break,
                Err(_) => self.0.status.store(POLLING, ORDERING),
            }
        }
    }
}

#[inline]
unsafe fn waker(task: *const Task) -> Waker {
    Waker::from_raw(RawWaker::new(
        task as *const (),
        &RawWakerVTable::new(clone_raw, wake_raw, wake_ref_raw, drop_raw),
    ))
}

#[inline]
unsafe fn clone_raw(this: *const ()) -> RawWaker {
    let task = clone_task(this as *const Task);
    RawWaker::new(
        Rc::into_raw(task.0) as *const (),
        &RawWakerVTable::new(clone_raw, wake_raw, wake_ref_raw, drop_raw),
    )
}

#[inline]
unsafe fn drop_raw(this: *const ()) {
    drop(task(this as *const Task))
}

#[inline]
unsafe fn wake_raw(this: *const ()) {
    let task = task(this as *const Task);
    let mut status = task.0.status.load(ORDERING);
    loop {
        match status {
            WAITING => {
                match task
                    .0
                    .status
                    .compare_exchange(WAITING, POLLING, ORDERING, ORDERING)
                {
                    Ok(_) => {
                        task.0.queue.send(clone_task(&*task.0)).unwrap();
                        break;
                    }
                    Err(cur) => status = cur,
                }
            }
            POLLING => {
                match task
                    .0
                    .status
                    .compare_exchange(POLLING, REPOLL, ORDERING, ORDERING)
                {
                    Ok(_) => break,
                    Err(cur) => status = cur,
                }
            }
            _ => break,
        }
    }
}

#[inline]
unsafe fn wake_ref_raw(this: *const ()) {
    let task = ManuallyDrop::new(task(this as *const Task));
    let mut status = task.0.status.load(ORDERING);
    loop {
        match status {
            WAITING => {
                match task
                    .0
                    .status
                    .compare_exchange(WAITING, POLLING, ORDERING, ORDERING)
                {
                    Ok(_) => {
                        task.0.queue.send(clone_task(&*task.0)).unwrap();
                        break;
                    }
                    Err(cur) => status = cur,
                }
            }
            POLLING => {
                match task
                    .0
                    .status
                    .compare_exchange(POLLING, REPOLL, ORDERING, ORDERING)
                {
                    Ok(_) => break,
                    Err(cur) => status = cur,
                }
            }
            _ => break,
        }
    }
}

#[inline]
unsafe fn task(future: *const Task) -> RcTask {
    RcTask(Rc::from_raw(future))
}

#[inline]
unsafe fn clone_task(future: *const Task) -> RcTask {
    let task = task(future);
    forget(task.clone());
    task
}
