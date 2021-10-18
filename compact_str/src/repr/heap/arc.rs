use std::sync::atomic::Ordering;
use std::{
    fmt,
    mem,
    slice,
    str,
};

use inner::{
    ArcStrInner,
    StrBuffer,
};
use ptr::OffsetPointer;

/// A soft limit on the amount of references that may be made to an `Arc`.
///
/// Going above this limit will abort your program (although not
/// necessarily) at _exactly_ `MAX_REFCOUNT + 1` references.
const MAX_REFCOUNT: usize = (isize::MAX) as usize;

#[repr(C)]
pub struct ArcStr {
    len: usize,
    ptr: OffsetPointer<ArcStrInner, StrBuffer>,
}

impl ArcStr {
    #[inline]
    pub fn new() -> Self {
        let ptr = ArcStrInner::new();
        // SAFETY: We just created the ArcStrInner and we know the pointer is valid
        let offset_ptr = unsafe { OffsetPointer::from_raw(ptr) };

        ArcStr {
            len: 0,
            ptr: offset_ptr,
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        let ptr = self.ptr.as_ref() as *const u8;
        let buffer = unsafe { slice::from_raw_parts(ptr, self.len) };
        unsafe { str::from_utf8_unchecked(buffer) }
    }

    #[inline(never)]
    unsafe fn drop_inner(&mut self) {
        ArcStrInner::dealloc(self.ptr.as_raw())
    }
}

impl Clone for ArcStr {
    fn clone(&self) -> Self {
        let old_count = unsafe { self.ptr.ref_count().fetch_add(1, Ordering::Relaxed) };

        if old_count > MAX_REFCOUNT {
            panic!("Program has gone wild, ref count > {}", MAX_REFCOUNT);
        }

        ArcStr {
            len: self.len,
            ptr: self.ptr,
        }
    }
}

impl Drop for ArcStr {
    fn drop(&mut self) {
        if unsafe { self.ptr.ref_count().fetch_sub(1, Ordering::Release) } != 1 {
            return;
        }
        std::sync::atomic::fence(Ordering::Acquire);
        unsafe { self.drop_inner() }
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
        let ptr = ArcStrInner::with_capacity(len);

        // SAFETY: we just created the ArcStrInner so we know it's valid
        let offset_ptr = unsafe {
            (*ptr).str_buffer.as_mut_ptr().copy_from(text.as_ptr(), len);
            OffsetPointer::from_raw(ptr)
        };

        ArcStr {
            len,
            ptr: offset_ptr,
        }
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

mod inner {
    use std::sync::atomic::{
        AtomicUsize,
        Ordering,
    };
    use std::{
        alloc,
        mem,
    };

    const UNKNOWN: usize = 0;
    pub type StrBuffer = [u8; UNKNOWN];

    #[repr(C)]
    pub struct ArcStrInner {
        ref_count: AtomicUsize,
        capacity: usize,
        pub str_buffer: StrBuffer,
    }

    impl ArcStrInner {
        pub fn new() -> *mut ArcStrInner {
            Self::with_capacity(0)
        }

        pub fn with_capacity(capacity: usize) -> *mut ArcStrInner {
            let ptr = unsafe { Self::alloc(capacity) };

            unsafe { (*ptr).ref_count = AtomicUsize::new(1) };
            unsafe { (*ptr).capacity = capacity };

            ptr
        }

        #[inline]
        pub fn ref_count(&self) -> usize {
            self.ref_count.load(Ordering::SeqCst)
        }

        #[inline]
        pub fn capacity(&self) -> usize {
            self.capacity
        }

        unsafe fn layout(capacity: usize) -> alloc::Layout {
            let buffer_layout = alloc::Layout::array::<u8>(capacity).unwrap();
            alloc::Layout::new::<Self>()
                .extend(buffer_layout)
                .unwrap()
                .0
                .pad_to_align()
        }

        pub unsafe fn alloc(capacity: usize) -> *mut ArcStrInner {
            let layout = Self::layout(capacity);
            let ptr = alloc::alloc(layout) as *mut ArcStrInner;
            if ptr.is_null() {
                alloc::handle_alloc_error(layout);
            }

            ptr
        }

        pub unsafe fn dealloc(ptr: *mut ArcStrInner) {
            let capacity = (*ptr).capacity;
            let layout = Self::layout(capacity);

            alloc::dealloc(ptr as *mut u8, layout);
        }
    }

    static_assertions::const_assert_eq!(mem::size_of::<ArcStrInner>(), 16);
}

mod ptr {
    use std::marker::PhantomData;
    use std::sync::atomic::AtomicUsize;
    use std::{
        mem,
        ptr,
    };

    use super::inner::{
        ArcStrInner,
        StrBuffer,
    };

    pub struct OffsetPointer<T, F> {
        ptr: ptr::NonNull<F>,
        _phantom: PhantomData<T>,
    }

    impl<T, F> OffsetPointer<T, F> {
        pub fn as_ref(&self) -> &F {
            unsafe { self.ptr.as_ref() }
        }

        pub fn as_ptr(&mut self) -> *mut F {
            self.ptr.as_ptr()
        }
    }

    impl OffsetPointer<ArcStrInner, StrBuffer> {
        pub unsafe fn from_raw(inner: *mut ArcStrInner) -> Self {
            const OFFSET: usize = mem::size_of::<AtomicUsize>() + mem::size_of::<usize>();
            let ptr = inner as *const u8;
            let str_buffer_ptr = ptr.offset(OFFSET as isize) as *mut StrBuffer;

            static_assertions::const_assert_eq!(mem::size_of::<ArcStrInner>(), OFFSET);

            let non_null = ptr::NonNull::new_unchecked(str_buffer_ptr);

            OffsetPointer {
                ptr: non_null,
                _phantom: PhantomData,
            }
        }

        pub unsafe fn as_raw(&mut self) -> *mut ArcStrInner {
            const OFFSET: isize =
                -(mem::size_of::<AtomicUsize>() as isize + mem::size_of::<usize>() as isize);
            let ptr = self.ptr.as_ptr() as *const u8;
            ptr.offset(OFFSET) as *mut ArcStrInner
        }

        #[inline]
        pub unsafe fn ref_count(&self) -> &AtomicUsize {
            const OFFSET: isize =
                -(mem::size_of::<AtomicUsize>() as isize + mem::size_of::<usize>() as isize);
            &*self.cast_offset::<AtomicUsize, OFFSET>()
        }

        // dead code for now
        // #[inline]
        // pub unsafe fn capacity(&self) -> usize {
        //     const OFFSET: isize = -(mem::size_of::<usize>() as isize);
        //     *self.cast_offset::<usize, OFFSET>()
        // }

        #[inline]
        fn cast_offset<T, const OFFSET: isize>(&self) -> *const T {
            let ptr = self.ptr.as_ptr() as *const u8;
            let offset_ptr = unsafe { ptr.offset(OFFSET) } as *const T;

            offset_ptr
        }
    }

    impl Copy for OffsetPointer<ArcStrInner, StrBuffer> {}

    impl Clone for OffsetPointer<ArcStrInner, StrBuffer> {
        fn clone(&self) -> Self {
            OffsetPointer {
                ptr: self.ptr,
                _phantom: PhantomData,
            }
        }
    }
}

static_assertions::const_assert_eq!(mem::size_of::<ArcStr>(), 16);
