// Copyright (C) 2017 1aim GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use nom::IResult;

use parser::helper::*;
use parser::consts;

pub fn phone_number(i: &str) -> IResult<&str, Number> {
	let (_, i)    = try_parse!(i, extract);
	let extension = consts::EXTN_PATTERN.captures(i);

	IResult::Done("", Number {
		value: extension.as_ref()
			.map(|c| &i[.. c.get(0).unwrap().start()])
			.unwrap_or(i)
			.into(),

		extension: extension.as_ref()
			.map(|c| c.get(2).unwrap().as_str())
			.map(Into::into),

		.. Default::default()
	})
}

#[cfg(test)]
mod test {
	use parser::natural;
	use parser::helper::*;

	#[test]
	fn phone_number() {
		assert_eq!(natural::phone_number("650 253 0000 extn. 4567").unwrap().1,
			Number {
				value:     "650 253 0000".into(),
				extension: Some("4567".into()),

				.. Default::default()
			});
	}
}