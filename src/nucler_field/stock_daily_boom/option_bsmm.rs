const AA1: f64 = 0.31938153;
const AA2: f64 = -0.356563782;
const AA3: f64 = 1.781477937;
const AA4: f64 = -1.821255978;
const AA5: f64 = 1.330274429;
const GAMMA_CDF: f64 = 0.2316419;
const PI: f64 = std::f64::consts::PI;

#[derive(Debug, PartialEq, Eq)]
pub enum OptFlag {
    Call,
    Put,
}

pub struct OptionStruct {
    // 可以通过struct.value的方式更改内容
    pub cp_flag: OptFlag,    // Call期权，Put期权
    pub now_price: f64,      // 当下资产价格
    pub strike_price: f64,   // 期权行权价格
    pub fluctuate: f64,      // 波动率
    pub risk_free_rate: f64, // 无风险利率，一般使用十年期国债利率
    pub now_time: f64,       // 当下时间，以年做为单位，未来可以转化为以天做为单位
    pub end_time: f64,       // 到期时间，格式同now_time
    pub division: f64,       // 分红
    d1: f64,
    d2: f64,
    tmp1: f64,
    tmp2: f64,
    tmp3: f64,
    duration_bsm: f64, // 公式中会用到的临时变量
    // 期权中的各种值
    pricer: f64, // 期权价格
    delta: f64,
    gamma: f64,
    kappa: f64,
    dual_gamma: f64, // Asset Value
    theta: f64,
    rho: f64,
    vega: f64,
    vomma: f64,
    psi: f64, // Time Value
}
impl Default for OptionStruct {
    fn default() -> Self {
        OptionStruct {
            cp_flag: OptFlag::Call, // 默认的new为看涨期权
            now_price: 100.25,
            strike_price: 120.50, // 价格
            fluctuate: 0.27342,
            risk_free_rate: 0.03, // 波动
            now_time: 0.1,
            end_time: 1.0,   // 时间
            division: 0.005, // 分红
            d1: 0.0,
            d2: 0.0,
            tmp1: 0.0,
            tmp2: 0.0,
            tmp3: 0.0,
            duration_bsm: 0.0, // 临时变量
            pricer: 0.0,       // 期权价格
            delta: 0.0,
            gamma: 0.0,
            kappa: 0.0,
            dual_gamma: 0.0, // Asset Value
            theta: 0.0,
            rho: 0.0,
            vega: 0.0,
            vomma: 0.0,
            psi: 0.0, // Time Value
        }
    }
}

impl OptionStruct {
    pub fn new(
        flag: OptFlag,
        now: f64,
        stk: f64,
        vix: f64,
        i10y: f64,
        tdy: f64,
        ddl: f64,
        dvs: f64,
    ) -> OptionStruct {
        OptionStruct {
            cp_flag: flag, // 默认的new为看涨期权
            now_price: now,
            strike_price: stk, // 价格
            fluctuate: vix,
            risk_free_rate: i10y, // 波动
            now_time: tdy,
            end_time: ddl, // 时间
            division: dvs, // 分红
            d1: 0.0,
            d2: 0.0,
            tmp1: 0.0,
            tmp2: 0.0,
            tmp3: 0.0,
            duration_bsm: 0.0, // 临时变量
            pricer: 0.0,       // 期权价格
            delta: 0.0,
            gamma: 0.0,
            kappa: 0.0,
            dual_gamma: 0.0, // Asset Value
            theta: 0.0,
            rho: 0.0,
            vega: 0.0,
            vomma: 0.0,
            psi: 0.0, // Time Value
        }
    }

    fn cdf_norm(x: &f64) -> f64 {
        let l: f64 = x.abs();
        let z: f64 = 1.0 / (1.0 + GAMMA_CDF * l);
        let aa0: f64 =
            AA1 * z + AA2 * z * z + AA3 * z * z * z + AA4 * z * z * z * z + AA5 * z.powf(5.0);
        let w: f64 = 1.0 - (1.0 / (2.0 * PI).sqrt()) * aa0 * (-0.5 * l * l).exp();
        if *x < 0.0 { 1.0 - w } else { w }
    }

    fn pdf_norm(x: &f64) -> f64 {
        let pdf_p1: f64 = (-0.5 * x.powf(2.0)).exp();
        let pdf_p2: f64 = (2.0 * PI).sqrt();
        pdf_p1 / pdf_p2
    }

    fn d1_d2_tmp(&mut self) {
        self.duration_bsm = self.end_time - self.now_time;
        let dtime: &f64 = &self.duration_bsm;
        let d1p1: f64 = (self.now_price / self.strike_price).ln();
        let d1p2: f64 = self.risk_free_rate - self.division + 0.5 * self.fluctuate.powf(2.0);
        let d1p3: f64 = self.fluctuate * dtime.sqrt();
        self.d1 = (d1p1 + d1p2 * dtime) / d1p3;
        self.d2 = self.d1 - d1p3;
        self.tmp2 = self.now_price * (-self.division * dtime).exp();
        if self.cp_flag == OptFlag::Call {
            let tmp1p1: f64 = self.division * Self::cdf_norm(&self.d1);
            let tmp1p2: f64 = self.fluctuate * (-0.5 * self.d1.powf(2.0)).exp();
            let tmp1p3: f64 = (8.0 * PI * dtime).sqrt();
            self.tmp1 = tmp1p1 - tmp1p2 / tmp1p3;
            let tmp3p1: f64 = self.fluctuate * self.strike_price;
            let tmp3p2: f64 = (-self.risk_free_rate * dtime).exp();
            self.tmp3 = tmp3p1 * Self::cdf_norm(&self.d2) * tmp3p2;
        } else {
            // cp_flag == OptFlag::Put
            let tmp1p1: f64 = self.division * Self::cdf_norm(&-self.d1);
            let tmp1p2: f64 = self.fluctuate * (-0.5 * self.d1.powf(2.0)).exp();
            let tmp1p3: f64 = (8.0 * PI * dtime).sqrt();
            self.tmp1 = tmp1p1 - tmp1p2 / tmp1p3;
            let tmp3p1: f64 = self.risk_free_rate * self.strike_price;
            let tmp3p2: f64 = (-self.risk_free_rate * dtime).exp();
            self.tmp3 = tmp3p1 * Self::cdf_norm(&-self.d2) * tmp3p2;
        }
    }

    fn g_d_vv(&mut self) {
        let dtime: &f64 = &self.duration_bsm;
        let gamma_p1: f64 = (-self.division * dtime).exp();
        let gamma_p2: f64 = self.fluctuate * self.now_price * dtime.sqrt();
        self.gamma = (gamma_p1 * Self::pdf_norm(&self.d1)) / gamma_p2;
        let d_g_p1: f64 = -self.risk_free_rate * dtime - 0.5 * self.d2.powf(2.0);
        let d_g_p2: f64 = d_g_p1.exp();
        let d_g_p3: f64 = self.fluctuate * self.strike_price;
        let d_g_p4: f64 = (2.0 * dtime * PI).sqrt();
        self.dual_gamma = d_g_p2 / (d_g_p3 * d_g_p4);
        let vega_p1: f64 = (dtime / (2.0 * PI)).sqrt();
        let vega_p2: f64 = -self.risk_free_rate * dtime - 0.5 * self.d2.powf(2.0);
        let vega_p3: f64 = vega_p2.exp();
        self.vega = vega_p1 * self.strike_price * vega_p3;
        self.vomma = self.vega * self.d1 * self.d2 / self.fluctuate;
    }

    fn price_and_others(&mut self) {
        let dtime: &f64 = &self.duration_bsm;
        if self.cp_flag == OptFlag::Call {
            let pp1: f64 = self.now_price * (-self.division * dtime).exp();
            let pp2: f64 = self.strike_price * (-self.risk_free_rate * dtime).exp();
            self.pricer = pp1 * Self::cdf_norm(&self.d1) - pp2 * Self::cdf_norm(&self.d2);
            self.delta = (-self.division * dtime).exp() * Self::cdf_norm(&self.d1);
            self.kappa = -(-self.risk_free_rate * dtime).exp() * Self::cdf_norm(&self.d2);
            self.theta = self.tmp1 * self.tmp2 - self.tmp3;
            let rho_p1: f64 = dtime * self.strike_price;
            let rho_p2: f64 = (-self.risk_free_rate * dtime).exp();
            self.rho = rho_p1 * rho_p2 * Self::cdf_norm(&self.d2);
            let psi_p1: f64 = -dtime * self.now_price;
            let psi_p2: f64 = (-self.division * dtime).exp();
            self.psi = psi_p1 * psi_p2 * Self::cdf_norm(&self.d1);
        } else {
            // cp_flag == "Put"
            let pp1: f64 = -self.now_price * (-self.division * dtime).exp();
            let pp2: f64 = self.strike_price * (-self.risk_free_rate * dtime).exp();
            self.pricer = pp1 * Self::cdf_norm(&-self.d1) + pp2 * Self::cdf_norm(&-self.d2);
            self.delta = -(-self.division * dtime).exp() * Self::cdf_norm(&-self.d1);
            self.kappa = (-self.risk_free_rate * dtime).exp() * Self::cdf_norm(&-self.d2);
            self.theta = self.tmp3 - self.tmp1 * self.tmp2;
            let rho_p1: f64 = -dtime * self.strike_price;
            let rho_p2: f64 = (-self.risk_free_rate * dtime).exp();
            self.rho = rho_p1 * rho_p2 * Self::cdf_norm(&-self.d2);
            let psi_p1: f64 = dtime * self.now_price;
            let psi_p2: f64 = (-self.division * dtime).exp();
            self.psi = psi_p1 * psi_p2 * Self::cdf_norm(&-self.d1);
        }
    }

    pub fn display_it(&self) {
        if self.pricer < (self.now_price * 0.005) {
            println!("Over 200x leverage, something else should be considered first");
        } else {
            println!(
                "\t资产价格从[{0}]到[{1}]的剩余时间为[{2}]个单位时间的[{3:?}]期权 ---> 理论价格为[{4}]",
                self.now_price, self.strike_price, self.duration_bsm, self.cp_flag, self.pricer
            );
            println!(
                "\t波动率 -> {0}%，无风险利率 -> {1}%，分红率 -> {2}%\t当前时间 -> {3}，到期时间 -> {4}",
                100.0 * self.fluctuate,
                100.0 * self.risk_free_rate,
                100.0 * self.division,
                self.now_time,
                self.end_time
            );
            println!(
                "\tdelta = {0}\tgamma = {1}\n\tkappa = {2}\tdual-gamma = {3}",
                self.delta, self.gamma, self.kappa, self.dual_gamma
            );
            println!(
                "\ttheta = {0}\trho = {1}\n\tvega = {2}\tvomma = {3}\tpsi = {4}",
                self.theta, self.rho, self.vega, self.vomma, self.psi
            );
        }
    }

    pub fn test_et_display(&mut self) {
        self.d1_d2_tmp();
        self.g_d_vv();
        self.price_and_others();
        self.display_it();
        /* use rand::Rng; */
        // let mut opt_ito = bsm_m::OptionStruct::default();
        // opt_ito.strike_price = rand::thread_rng().gen_range(50.0..250.0);
        // opt_ito.fluctuate = rand::thread_rng().gen_range(0.0..1.0);
        // opt_ito.test_et_display();
    }
}
