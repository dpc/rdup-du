.PHONY: all test run clean doc

OUT = bin
SRC = src
DOC = doc

PROJECT=rdu

RCFLAGS_LIBS=
RCFLAGS = $(RCFLAGS_LIBS)

ifdef DEBUG
RCFLAGS += -g
else
RCFLAGS += -O
endif

LOG_LEVEL=debug

all: $(OUT)/$(PROJECT)

clean:
	rm -f $(OUT)/$(PROJECT)

$(OUT)/$(PROJECT): $(SRC)/*.rs
	mkdir -p $(OUT)
	rustc $(RCFLAGS) --out-dir=$(OUT) -o $(OUT)/$(PROJECT)  src/main.rs

run: $(OUT)/$(PROJECT)
	LOG=$(LOG_LEVEL) $(OUT)/$(PROJECT)

doc:
	rustdoc $(RCFLAGS_LIBS) --output=$(DOC) src/main.rs
