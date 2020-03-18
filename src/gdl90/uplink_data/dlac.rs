
enum State { 
	StepOne, 
	StepTwo(u8),
	StepThree(u8),
}

pub struct Decoder {
	state:State,
	string:String,
}

fn u8_to_char(x:u8) -> char { match x {
	 0 => '_',  1 => 'A',  2 => 'B',  3 => 'C',  4 => 'D',  5 => 'E',  6 => 'F',  7 => 'G',  
	 8 => 'H',  9 => 'I', 10 => 'J', 11 => 'K', 12 => 'L', 13 => 'M', 14 => 'N', 15 => 'O',  
	16 => 'P', 17 => 'Q', 18 => 'R', 19 => 'S', 20 => 'T', 21 => 'U', 22 => 'V', 23 => 'W',  
	24 => 'X', 25 => 'Y', 26 => 'Z', 27 => '_', 28 => '_', 29 => '_', 30 => '_', 31 => '_',  
	32 => ' ', 33 => '_', 34 => '_', 35 => '_', 36 => '_', 37 => '_', 38 => '_', 39 => '_',  
	40 => '_', 41 => '_', 42 => '_', 43 => '_', 44 => '_', 45 => '_', 46 => '_', 47 => '/',  
	48 => '0', 49 => '1', 50 => '2', 51 => '3', 52 => '4', 53 => '5', 54 => '6', 55 => '7',  
	56 => '8', 57 => '9', 58 => '_', 59 => '_', 60 => '_', 61 => '_', 62 => '_', 63 => '_',
	_  => '_',
}}

impl Decoder {

	pub fn new() -> Decoder { Decoder{ state: State::StepOne, string: String::new() } }

	pub fn next(&mut self, next_byte:u8) {
		let next_state:State = match self.state {
			State::StepOne => {
				self.string.push(u8_to_char(next_byte >> 2));
				State::StepTwo(next_byte % 4)
			},
			State::StepTwo(remainder) => {
				self.string.push(u8_to_char((remainder*16) + (next_byte >> 4)));
				State::StepThree(next_byte % 16)
			},
			State::StepThree(remainder) => {
				self.string.push(u8_to_char((remainder*4)  + (next_byte >> 6)));
				self.string.push(u8_to_char(next_byte % 64));
				State::StepOne
			}
		};

		self.state = next_state;
	}

	pub fn get_result(&mut self) -> String {
		let ans = self.string.clone();
		self.state = State::StepOne;
		self.string.clear();
		ans
	}
}

