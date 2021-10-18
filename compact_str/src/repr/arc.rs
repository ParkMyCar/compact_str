use std::alloc;
use std::fmt;
use std::mem;
use std::ops::Deref;
use std::ptr;
use std::slice;
use std::str;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const UNKNOWN: usize = 0;
type StrBuffer = [u8; UNKNOWN];

const REF_COUNT_AND_CAPACITY_SIZE: isize =
    (mem::size_of::<AtomicUsize>() + mem::size_of::<usize>()) as isize;

#[repr(C)]
#[derive(Clone)]
pub struct ArcStr {
    len: usize,
    ptr: ArcStrPtr,
}

impl ArcStr {
    #[inline]
    pub fn new() -> Self {
        let ptr = ArcStrInner::new();

        ArcStr { len: 0, ptr }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        let ptr = unsafe { self.ptr.as_str_buffer().as_ptr() };
        let buffer = unsafe { slice::from_raw_parts(ptr, self.len) };
        unsafe { str::from_utf8_unchecked(buffer) }
    }
}

impl fmt::Debug for ArcStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl From<&str> for ArcStr {
    fn from(text: &str) -> Self {
        let len = text.len();
        let mut ptr = ArcStrInner::with_capacity(len);
        unsafe {
            ptr.as_mut_str_buffer()
                .as_mut_ptr()
                .copy_from(text.as_ptr(), len)
        };

        ArcStr { len, ptr }
    }
}

impl Deref for ArcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

#[cfg(test)]
mod test {
    use super::ArcStr;

    #[test]
    fn test_sanity() {
        let example = "hello world!";
        let arc_str = ArcStr::from(example);

        assert_eq!(arc_str.as_str(), example);
        assert_eq!(arc_str.len(), example.len());
    }
}

#[repr(C)]
struct ArcStrInner {
    ref_count: AtomicUsize,
    capacity: usize,
    str_buffer: StrBuffer,
}

impl ArcStrInner {
    pub fn new() -> ArcStrPtr {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> ArcStrPtr {
        let ptr = unsafe { Self::alloc(capacity) };

        unsafe { (*ptr).ref_count = AtomicUsize::new(1) };
        unsafe { (*ptr).capacity = capacity };

        unsafe { ArcStrPtr::from_arc_str_inner(ptr) }
    }

    unsafe fn layout(capacity: usize) -> alloc::Layout {
        let buffer_layout = alloc::Layout::array::<u8>(capacity).unwrap();
        alloc::Layout::new::<Self>()
            .extend(buffer_layout)
            .unwrap()
            .0
            .pad_to_align()
    }

    unsafe fn alloc(capacity: usize) -> *mut ArcStrInner {
        let layout = Self::layout(capacity);
        let ptr = alloc::alloc(layout) as *mut ArcStrInner;
        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }

        ptr
    }

    unsafe fn dealloc(ptr: *mut ArcStrInner) {
        let capacity = (*ptr).capacity;
        let layout = Self::layout(capacity);

        alloc::dealloc(ptr as *mut u8, layout);
    }
}

// TODO(parkertimmerman): Once the unstable feature `const_ptr_offset` has become stabilized, we can
// change this struct to be:
// ```
// pub struct ArcStrPtr {
//     ptr: ptr::NonNull<ArcStrInner>,
// }
// ```
// and implement `const fn`s that return pointers to the inner fields, calculating the offsets at
// compile time.
#[repr(transparent)]
pub struct ArcStrPtr {
    ptr: ptr::NonNull<StrBuffer>,
}

impl ArcStrPtr {
    #[inline]
    unsafe fn from_arc_str_inner(ptr: *mut ArcStrInner) -> Self {
        let ptr = ptr as *const u8;
        let str_buffer_ptr = ptr.offset(REF_COUNT_AND_CAPACITY_SIZE) as *mut StrBuffer;

        ArcStrPtr {
            ptr: ptr::NonNull::new_unchecked(str_buffer_ptr),
        }
    }

    #[inline]
    unsafe fn as_arc_str_inner(&self) -> &ArcStrInner {
        let ptr = self.ptr.as_ptr() as *const u8;
        let offset_ptr = ptr.offset(-REF_COUNT_AND_CAPACITY_SIZE) as *const ArcStrInner;

        &*offset_ptr
    }

    #[inline]
    unsafe fn as_str_buffer(&self) -> &StrBuffer {
        self.ptr.as_ref()
    }

    #[inline]
    unsafe fn as_mut_str_buffer(&mut self) -> &mut StrBuffer {
        self.ptr.as_mut()
    }

    #[inline(never)]
    fn slow_drop(&mut self) {
        unsafe { ArcStrInner::dealloc(self.as_arc_str_inner() as *const ArcStrInner as *mut _) }
    }
}

impl Clone for ArcStrPtr {
    fn clone(&self) -> Self {
        unsafe {
            let _old_count = self
                .as_arc_str_inner()
                .ref_count
                .fetch_add(1, Ordering::Relaxed);
        }
        ArcStrPtr { ptr: self.ptr }
    }
}

impl Drop for ArcStrPtr {
    fn drop(&mut self) {
        let inner = unsafe { self.as_arc_str_inner() };
        if inner.ref_count.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }

        std::sync::atomic::fence(Ordering::Acquire);

        self.slow_drop();
    }
}

static_assertions::const_assert_eq!(
    mem::size_of::<AtomicUsize>() + mem::size_of::<usize>(),
    REF_COUNT_AND_CAPACITY_SIZE as usize
);

static_assertions::const_assert_eq!(mem::size_of::<ArcStrInner>(), 16);
static_assertions::const_assert_eq!(mem::size_of::<ArcStrPtr>(), 8);
static_assertions::const_assert_eq!(mem::size_of::<ArcStr>(), 16);
