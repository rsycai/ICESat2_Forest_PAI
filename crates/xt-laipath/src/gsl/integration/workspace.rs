#[derive(Debug, Clone)]
pub struct GslIntegrationWorkspace {
    pub limit: usize,
    pub size: usize,
    pub nrmax: usize,
    pub i: usize,
    pub maximum_level: usize,
    pub alist: Vec<f64>,
    pub blist: Vec<f64>,
    pub rlist: Vec<f64>,
    pub elist: Vec<f64>,
    pub order: Vec<usize>,
    pub level: Vec<usize>,
}

impl GslIntegrationWorkspace {
    #[inline]
    pub fn new(limit: usize) -> Self {
        GslIntegrationWorkspace {
            limit,
            size: 0,
            nrmax: 0,
            i: 0,
            maximum_level: 0,
            alist: vec![0.0; limit], // Initialize to the requested size.
            blist: vec![0.0; limit],
            rlist: vec![0.0; limit],
            elist: vec![0.0; limit],
            order: vec![0; limit],
            level: vec![0; limit],
        }
    }

    #[inline]
    pub fn initialise(&mut self, a: f64, b: f64) {
        self.size = 0;
        self.nrmax = 0;
        self.i = 0;
        self.alist[0] = a;
        self.blist[0] = b;
        self.rlist[0] = 0.0;
        self.elist[0] = 0.0;
        self.order[0] = 0;
        self.level[0] = 0;

        self.maximum_level = 0;
    }

    #[inline]
    pub fn append_interval(&mut self, a1: f64, b1: f64, area1: f64, error1: f64) {
        let i_new = self.size;

        self.alist[i_new] = a1;
        self.blist[i_new] = b1;
        self.rlist[i_new] = area1;
        self.elist[i_new] = error1;
        self.order[i_new] = i_new;
        self.level[i_new] = 0;

        self.size += 1;
    }

    #[inline]
    pub fn sort_results(&mut self) {
        let elist = &self.elist;
        let order = &mut self.order;

        let nint = self.size;

        order[..nint].sort_by(|&a, &b| elist[b].total_cmp(&elist[a]));
        self.i = order[0];
    }

    #[inline]
    pub fn reset_nrmax(&mut self) {
        self.nrmax = 0;
        self.i = self.order[0];
    }

    #[inline]
    pub fn retrieve(&self, a: &mut f64, b: &mut f64, r: &mut f64, e: &mut f64) {
        let i = self.i;
        let alist = &self.alist;
        let blist = &self.blist;
        let rlist = &self.rlist;
        let elist = &self.elist;

        *a = alist[i];
        *b = blist[i];
        *r = rlist[i];
        *e = elist[i];
    }

    #[inline]
    pub fn update(
        &mut self,
        a1: f64,
        b1: f64,
        area1: f64,
        error1: f64,
        a2: f64,
        b2: f64,
        area2: f64,
        error2: f64,
    ) {
        let alist = &mut self.alist;
        let blist = &mut self.blist;
        let rlist = &mut self.rlist;
        let elist = &mut self.elist;
        let level = &mut self.level;

        let i_max = self.i;
        let i_new = self.size;

        let new_level = level[i_max] + 1;

        if error2 > error1 {
            alist[i_max] = a2; // blist[maxerr] is already == b2
            rlist[i_max] = area2;
            elist[i_max] = error2;
            level[i_max] = new_level;

            alist[i_new] = a1;
            blist[i_new] = b1;
            rlist[i_new] = area1;
            elist[i_new] = error1;
            level[i_new] = new_level;
        } else {
            blist[i_max] = b1; // alist[maxerr] is already == a1
            rlist[i_max] = area1;
            elist[i_max] = error1;
            level[i_max] = new_level;

            alist[i_new] = a2;
            blist[i_new] = b2;
            rlist[i_new] = area2;
            elist[i_new] = error2;
            level[i_new] = new_level;
        }

        self.size += 1;

        if new_level > self.maximum_level {
            self.maximum_level = new_level;
        }

        self.qpsrt();
    }

    #[inline]
    fn qpsrt(&mut self) {
        let last = self.size - 1;
        let limit = self.limit;

        let elist = &self.elist;
        let order = &mut self.order;

        let mut i;
        let mut k: isize;

        let mut i_nrmax = self.nrmax;
        let mut i_maxerr = order[i_nrmax];

        if last < 2 {
            order[0] = 0;
            order[1] = 1;
            self.i = i_maxerr;
            return;
        }

        let errmax: f64 = elist[i_maxerr];

        while i_nrmax > 0 && errmax > elist[order[i_nrmax - 1]] {
            order[i_nrmax] = order[i_nrmax - 1];
            // self.nrmax = self.nrmax.saturating_sub(1);
            // self.nrmax -= 1;
            i_nrmax -= 1;
        }

        let top = if last < (limit / 2 + 2) {
            last
        } else {
            limit - last + 1
        };

        i = i_nrmax + 1;

        while i < top && errmax < elist[order[i]] {
            order[i - 1] = order[i];
            i += 1;
        }

        order[i - 1] = i_maxerr;

        let errmin: f64 = elist[last];
        k = top as isize - 1;

        while k > i as isize - 2 && errmin >= elist[order[k as usize]] {
            order[(k + 1) as usize] = order[k as usize];
            k -= 1;
        }

        order[(k + 1) as usize] = last;

        i_maxerr = order[i_nrmax];

        self.i = i_maxerr;
        self.nrmax = i_nrmax;
    }

    #[inline]
    pub fn large_interval(&self) -> i32 {
        let i = self.i;
        let level = &self.level;

        if level[i] < self.maximum_level { 1 } else { 0 }
    }

    #[inline]
    pub fn increase_nrmax(&mut self) -> i32 {
        let id = self.nrmax;
        let last = self.size - 1;
        let jupbnd = if last > (1 + self.limit / 2) {
            self.limit + 1 - last
        } else {
            last
        };

        for _ in id..=jupbnd {
            let i_max = self.order[self.nrmax];
            self.i = i_max;

            if self.level[i_max] < self.maximum_level {
                return 1;
            }

            self.nrmax += 1;
        }

        0
    }

    #[inline]
    pub fn sum_results(&self) -> f64 {
        let rlist = &self.rlist;
        let n = self.size;

        rlist[0..n].iter().sum()
    }

    #[inline]
    pub fn set_initial_result(&mut self, result: f64, error: f64) {
        self.size = 1;
        self.rlist[0] = result;
        self.elist[0] = error;
    }
}
