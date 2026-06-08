EXEC_DIR := target/release

STEPS := $(EXEC_DIR)/step0_repl \
		 $(EXEC_DIR)/step1_read_print \
		 $(EXEC_DIR)/step2_eval \
		 $(EXEC_DIR)/step3_env \
		 $(EXEC_DIR)/step4_if_fn_do \
		 $(EXEC_DIR)/step5_tco \
		 $(EXEC_DIR)/step6_file \
		 $(EXEC_DIR)/step7_quote \
		 $(EXEC_DIR)/step8_macros \
		 $(EXEC_DIR)/step9_try \
		 $(EXEC_DIR)/stepA_mal

$(STEPS): $(EXEC_DIR)/%: %.rs
	cargo build --release --bin $*

.PHONY: clean

clean:
	rm -rf target/
