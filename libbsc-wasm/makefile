SHELL = /bin/sh

CC = emcc
AR = emar
RANLIB = emranlib

CFLAGS = -Wall

# Comment out CFLAGS line below for compatability mode for 32bit file sizes
# (less than 2GB) and systems that have compilers that treat int as 64bit
# natively (ie: modern AIX)
#CFLAGS += -D_LARGEFILE64_SOURCE -D_FILE_OFFSET_BITS=64

# Comment out CFLAGS line below to disable optimizations
CFLAGS += -O3 -fomit-frame-pointer -fstrict-aliasing -ffast-math
COFLAGS = -Wall -O3 -fomit-frame-pointer -fstrict-aliasing -ffast-math
CFLAGS += -sEXPORTED_FUNCTIONS=_bsc_compress,_bsc_init,_bsc_store,_bsc_block_info,_bsc_decompress,_malloc,_free -sENVIRONMENT=web -sMODULARIZE -sINITIAL_MEMORY=64MB -sSTACK_SIZE=512KB --no-entry -sINVOKE_RUN=0 -sEXIT_RUNTIME=0 -sMALLOC=emmalloc
#COFLAGS = -g -Wall -fsanitize=address
#CFLAGS += -g -sEXPORTED_FUNCTIONS=_bsc_compress,_bsc_init,_bsc_store,_bsc_block_info,_bsc_decompress,_malloc,_free -sENVIRONMENT=web -sMODULARIZE  -sINITIAL_MEMORY=134217728 -sASSERTIONS -g -sSTACK_SIZE=8MB -sSTACK_OVERFLOW_CHECK=2 --no-entry -sINVOKE_RUN=0 -sEXIT_RUNTIME=0 -sMALLOC=emmalloc-memvalidate -fsanitize=address

# Comment out CFLAGS line below to disable AVX2 instruction set (performance will suffer)
#CFLAGS += -mavx2

# Comment out CFLAGS line below to disable OpenMP optimizations
#CFLAGS += -fopenmp -DLIBBSC_OPENMP_SUPPORT

# Comment out CFLAGS line below to enable debug output
CFLAGS += -DNDEBUG

# Comment out CFLAGS line below to disable Sort Transform
#CFLAGS += -DLIBBSC_SORT_TRANSFORM_SUPPORT

# Comment out CFLAGS line below to disable unaligned memory access
CFLAGS += -DLIBBSC_ALLOW_UNALIGNED_ACCESS

# Where you want bsc installed when you do 'make install'
PREFIX = /usr

OBJS = \
       adler32.o       \
       libsais.o       \
       coder.o         \
       qlfc.o          \
       qlfc_model.o    \
       detectors.o     \
       preprocessing.o \
       libbsc.o        \
       lzp.o           \
       platform.o      \
       st.o            \
       bwt.o           \

all: libbsc.a bsc

bsc: libbsc.a bsc.cpp
	$(CC) $(CFLAGS) bsc.cpp -o bsc -L. -lbsc
	
libbsc: libbsc.a
	$(CC) $(CFLAGS) libbsc.a -o bsc.js -L.

libbsc.a: $(OBJS)
	rm -f libbsc.a
	$(AR) cq libbsc.a $(OBJS)
	@if ( test -f $(RANLIB) -o -f /usr/bin/ranlib -o \
		-f /bin/ranlib -o -f /usr/ccs/bin/ranlib ) ; then \
		echo $(RANLIB) libbsc.a ; \
		$(RANLIB) libbsc.a ; \
	fi

install: libbsc.a bsc
	if ( test ! -d $(DESTDIR)$(PREFIX)/bin ) ; then mkdir -p $(DESTDIR)$(PREFIX)/bin ; fi
	if ( test ! -d $(DESTDIR)$(PREFIX)/lib ) ; then mkdir -p $(DESTDIR)$(PREFIX)/lib ; fi
	if ( test ! -d $(DESTDIR)$(PREFIX)/include ) ; then mkdir -p $(DESTDIR)$(PREFIX)/include ; fi
	cp -f bsc $(DESTDIR)$(PREFIX)/bin/bsc
	chmod a+x $(DESTDIR)$(PREFIX)/bin/bsc
	cp -f libbsc/libbsc.h $(DESTDIR)$(PREFIX)/include
	chmod a+r $(DESTDIR)$(PREFIX)/include/libbsc.h
	cp -f libbsc.a $(DESTDIR)$(PREFIX)/lib
	chmod a+r $(DESTDIR)$(PREFIX)/lib/libbsc.a

clean:
	rm -f *.o libbsc.a bsc.wasm bsc.js

adler32.o: libbsc/adler32/adler32.cpp
	$(CC) $(COFLAGS) -c libbsc/adler32/adler32.cpp

libsais.o: libbsc/bwt/libsais/libsais.c
	$(CC) $(COFLAGS) -c libbsc/bwt/libsais/libsais.c

coder.o: libbsc/coder/coder.cpp
	$(CC) $(COFLAGS) -c libbsc/coder/coder.cpp

qlfc.o: libbsc/coder/qlfc/qlfc.cpp
	$(CC) $(COFLAGS) -c libbsc/coder/qlfc/qlfc.cpp

qlfc_model.o: libbsc/coder/qlfc/qlfc_model.cpp
	$(CC) $(COFLAGS) -c libbsc/coder/qlfc/qlfc_model.cpp

detectors.o: libbsc/filters/detectors.cpp
	$(CC) $(COFLAGS) -c libbsc/filters/detectors.cpp

preprocessing.o: libbsc/filters/preprocessing.cpp
	$(CC) $(COFLAGS) -c libbsc/filters/preprocessing.cpp

libbsc.o: libbsc/libbsc/libbsc.cpp
	$(CC) $(COFLAGS) -c libbsc/libbsc/libbsc.cpp

lzp.o: libbsc/lzp/lzp.cpp
	$(CC) $(COFLAGS) -c libbsc/lzp/lzp.cpp

platform.o: libbsc/platform/platform.cpp
	$(CC) $(COFLAGS) -c libbsc/platform/platform.cpp

st.o: libbsc/st/st.cpp
	$(CC) $(COFLAGS) -c libbsc/st/st.cpp

bwt.o: libbsc/bwt/bwt.cpp
	$(CC) $(COFLAGS) -c libbsc/bwt/bwt.cpp
