mod project {
    static UNI_OWNER: &'static str = "Ruu";
    const RU_PI: f64 = std::f64::consts::PI;

    #[tokio::test]
    #[ignore]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        // 变量，元祖，断言，类型转换
        {
            let basic_tf: (bool, bool) = (true, false);
            let basic_num: (isize, isize, isize) = (1, 0, -1);
            let basic_str: (&str, String) = ("good", String::from("bad"));
            assert_eq!(0, basic_num.0 - basic_tf.0 as isize);
        }

        // 数组，切片，迭代器，解引用
        {
            let basic_array: [isize; 6] = [0, 1, 2, 3, 4, 5];
            let basic_slice: &[isize] = &basic_array[2..5];
            let bs_change: Vec<isize> = basic_slice.iter().map(|&x| x - 1).collect::<Vec<isize>>();
            assert_eq!(&basic_array[1..4], bs_change.as_slice()); // std::ops::Deref
        }

        // 枚举，结构体，嵌套
        {
            enum HomeFish {
                BigRedFish = 0xff0000,
                SmallBlueFish = 0x0000ff,
            }
            struct HomeRiver {
                location: String,
                fish: HomeFish,
            }
            let myhome = HomeRiver {
                location: String::from("Northland"),
                fish: HomeFish::BigRedFish,
            };
            assert_eq!(0xff0000, myhome.fish as u32);
        }

        // 链表，impl，奇怪的断言
        {
            enum CookStuff {
                Cons(String, Box<CookStuff>),
                Nil,
            }
            impl CookStuff {
                fn new() -> CookStuff {
                    CookStuff::Nil
                }
                fn prepend(self, element: &str) -> CookStuff {
                    CookStuff::Cons(String::from(element), Box::new(self))
                }
                fn len(&self) -> u32 {
                    match *self {
                        CookStuff::Cons(_, ref tail) => 1 + tail.len(),
                        CookStuff::Nil => 0,
                    }
                }
                fn stringify(&self) -> String {
                    match self {
                        CookStuff::Cons(head, tail) => {
                            format!("{}, {}", head, tail.stringify())
                        }
                        CookStuff::Nil => "Nil".to_string(),
                    }
                }
            }
            assert_eq!(
                "1, Meat, Salt, Nil",
                CookStuff::new()
                    .prepend("Salt")
                    .prepend("Meat")
                    .prepend(CookStuff::new().prepend("Ok").len().to_string().as_str())
                    .stringify()
                    .as_str(),
            );
        }

        // try from, try into
        {
            #[derive(Debug, PartialEq)]
            struct PrimeNumber(u64);
            impl TryFrom<u64> for PrimeNumber {
                type Error = String;

                fn try_from(value: u64) -> Result<Self, Self::Error> {
                    let non_prime_number_case: String = "not a prime number".to_string();
                    if value == 0 || value == 1 {
                        return Err(non_prime_number_case);
                    };

                    let sp: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
                    if sp.contains(&value) {
                        return Ok(PrimeNumber(value));
                    }
                    if sp.iter().any(|&p| value % p == 0) {
                        return Err(non_prime_number_case);
                    }

                    let sqrt_value = (value as f64).sqrt() as u64;
                    if (sp.last().unwrap() + 1..=sqrt_value).any(|i| value % i == 0) {
                        return Err(non_prime_number_case);
                    }
                    Ok(PrimeNumber(value))
                }
            }

            match PrimeNumber::try_from(u64::MAX - 1) {
                Ok(PrimeNumber(p)) => println!("Prime number: {}", p),
                Err(e) => assert_eq!("not a prime number", e.as_str()),
            }

            let result: Result<PrimeNumber, String> = 37u64.try_into();
            assert_eq!(result, Ok(PrimeNumber(37)));
        }

        // https://rustwiki.org/zh-CN/rust-by-example/conversion/try_from_try_into.html

        Ok(())
    }
}
