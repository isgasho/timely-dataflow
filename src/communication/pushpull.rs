use std::mem;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::mpsc::{Sender, Receiver};
use core::marker::PhantomData;

use communication::Observer;


pub trait Pushable<T> { fn push(&mut self, data: T); }        // like observer
pub trait Pullable<T> { fn pull(&mut self) -> Option<T>; }    // like iterator

impl<T> Pushable<T> for Vec<T> { fn push(&mut self, data: T) { self.push(data); } }
impl<T> Pullable<T> for Vec<T> { fn pull(&mut self) -> Option<T> { self.pop() } }

impl<T:Send> Pushable<T> for Sender<T> { fn push(&mut self, data: T) { self.send(data).ok().expect("send error"); } }
impl<T:Send> Pullable<T> for Receiver<T> { fn pull(&mut self) -> Option<T> { self.try_recv().ok() }}

impl<T, P: ?Sized + Pushable<T>> Pushable<T> for Box<P> { fn push(&mut self, data: T) { (**self).push(data); } }
impl<T, P: ?Sized + Pullable<T>> Pullable<T> for Box<P> { fn pull(&mut self) -> Option<T> { (**self).pull() } }

impl<T, P: Pushable<T>> Pushable<T> for Rc<RefCell<P>> { fn push(&mut self, data: T) { self.borrow_mut().push(data); } }
impl<T, P: Pullable<T>> Pullable<T> for Rc<RefCell<P>> { fn pull(&mut self) -> Option<T> { self.borrow_mut().pull() } }

pub struct PushableObserver<T:Send, D:Send+Clone, P: Pushable<(T, Vec<D>)>> {
    pub data:       Vec<D>,
    pub pushable:   P,
    pub phantom:    PhantomData<T>,
}

impl<T:Send+Clone, D:Send+Clone, P: Pushable<(T, Vec<D>)>> Observer for PushableObserver<T,D,P> {
    type Time = T;
    type Data = D;
    #[inline(always)] fn open(&mut self,_time: &T) { }
    #[inline(always)] fn show(&mut self, data: &D) { self.give(data.clone()); }
    #[inline(always)] fn give(&mut self, data:  D) { self.data.push(data); }
    #[inline(always)] fn shut(&mut self, time: &T) {
        if self.data.len() > 0 {
            self.pushable.push((time.clone(), mem::replace(&mut self.data, Vec::new())));
        }
    }
}