#include <iostream>

#include "dng_image_writer.h"
#include "dng_color_space.h"
#include "rawConverter.h"


extern "C" void buildDNG(unsigned short * image_data, unsigned short int width, unsigned short int height, const char * out_file, const char * exif_raw_file) {
    RawConverter converter;
    std::cout << "Reading exif from '" << exif_raw_file << "'..." << std::endl;
    converter.openRawFile(exif_raw_file);
    converter.buildNegative("", image_data, width, height);
    converter.renderImage();
    converter.renderPreviews();
    converter.writeDng(out_file);
}
