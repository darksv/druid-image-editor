
#[derive(Clone)]
pub(crate) struct Matrix<T> {
    data: Vec<T>,
    width: u32,
    height: u32,
}

impl<T> Matrix<T> {
    pub(crate) fn new(width: u32, height: u32) -> Self
        where T: Default + Copy {
        Matrix {
            width,
            height,
            data: vec![Default::default(); width as usize * height as usize],
        }
    }

    #[inline]
    pub(crate) fn width(&self) -> u32 { self.width }
    #[inline]
    pub(crate) fn height(&self) -> u32 { self.height }

    #[inline]
    pub(crate) fn get(&self, x: u32, y: u32) -> T where T: Copy {
        self.data[y as usize * self.width as usize + x as usize]
    }

    #[inline]
    pub(crate) fn set(&mut self, x: u32, y: u32, value: T) where T: Copy {
        self.data[y as usize * self.width as usize + x as usize] = value;
    }

    #[inline]
    pub(crate) fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }

    #[inline]
    pub(crate) fn as_slice_mut(&mut self) -> &mut [T] {
        self.data.as_mut_slice()
    }

    pub(crate) fn view(&self, x: u32, y: u32, width: u32, height: u32) -> View<'_, T> {
        View {
            buffer: &self.data,
            x,
            y,
            width,
            height,
        }
    }

    pub(crate) fn as_view(&self) -> View<'_, T> {
        View {
            buffer: &self.data,
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
        }
    }

    pub(crate) fn view_mut(&mut self, x: u32, y: u32, width: u32, height: u32) -> ViewMut<'_, T> {
        ViewMut {
            buffer: &mut self.data,
            x,
            y,
            width,
            height,
        }
    }

    pub(crate) fn as_view_mut(&mut self) -> ViewMut<'_, T> {
        ViewMut {
            buffer: &mut self.data,
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
        }
    }
}

pub(crate) struct View<'a, T> {
    buffer: &'a [T],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl<'a, T> View<'a, T> {
    pub(crate) fn new(buffer: &'a [T], x: u32, y: u32, width: u32, height: u32) -> Self {
        View {
            buffer,
            x,
            y,
            width,
            height,
        }
    }

    #[inline]
    pub(crate) fn width(&self) -> u32 { self.width }

    #[inline]
    pub(crate) fn height(&self) -> u32 { self.height }

    #[inline]
    pub(crate) fn get(&self, x: u32, y: u32) -> T where T: Copy {
        self.buffer[(self.y + y) as usize * self.width as usize + (self.x + x) as usize]
    }

    #[inline]
    pub(crate) fn as_slice(&self) -> Option<&'a [T]> {
        if self.x == 0 && self.y == 0 && self.width * self.height == self.buffer.len() as u32 {
            Some(self.buffer)
        } else {
            None
        }
    }

    pub(crate) fn to_matrix(&self) -> Matrix<T> where T: Copy + Default {
        let mut matrix = Matrix::new(self.width, self.height);
        matrix.as_slice_mut().copy_from_slice(self.as_slice().expect("whole buffer view"));
        matrix
    }
}

pub(crate) struct ViewMut<'a, T> {
    buffer: &'a mut [T],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl<'a, T> ViewMut<'a, T> {
    pub(crate) fn new(buffer: &'a mut [T], x: u32, y: u32, width: u32, height: u32) -> Self {
        ViewMut {
            buffer,
            x,
            y,
            width,
            height,
        }
    }

    #[inline]
    pub(crate) fn width(&self) -> u32 { self.width }

    #[inline]
    pub(crate) fn height(&self) -> u32 { self.height }

    #[inline]
    pub(crate) fn get(&self, x: u32, y: u32) -> T where T: Copy {
        self.buffer[(self.y + y) as usize * self.width as usize + (self.x + x) as usize]
    }

    #[inline]
    pub(crate) fn set(&mut self, x: u32, y: u32, value: T) where T: Copy {
        self.buffer[(self.y + y) as usize * self.width as usize + (self.x + x) as usize] = value;
    }
}
