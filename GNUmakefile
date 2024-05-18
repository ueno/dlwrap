# SPDX-License-Identifier: Apache-2.0

generate-fixtures::
	cargo run -- --input fixtures/clock_gettime.h \
		     --output-dir fixtures/out \
		     --symbol clock_gettime \
		     --prefix cgwrap \
		     --loader-basename cgwrap
