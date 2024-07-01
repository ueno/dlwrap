# SPDX-License-Identifier: Apache-2.0

generate-fixtures::
	cargo run -- --input fixtures/clock_gettime.h \
		     --output-dir fixtures/out \
		     --symbol clock_gettime \
		     --prefix cgwrap \
		     --include "<time.h>" \
		     --loader-basename cgwrap
	cargo run -- --input fixtures/array.h \
		     --output-dir fixtures/out \
		     --symbol compress \
		     --prefix array \
		     --loader-basename array
