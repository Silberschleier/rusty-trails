#include <iostream>

#include "dng_image_writer.h"
#include "dng_color_space.h"
#include "rawConverter.h"


extern "C" void buildDNG(unsigned short * image_data, unsigned short int width, unsigned short int height, const char * out_file) {
    std::cout << "Writing image of height " << height << " and width " << width << " to " << out_file << " ..." << std::endl;

    RawConverter converter;
    converter.openRawFile("./Lapse_007/2017-11-03_233129_Canon EOS 6D_2708_IMG_4437.CR2");
    converter.buildNegative("", image_data, width, height);
    converter.renderImage();
    converter.renderPreviews();
    converter.writeDng(out_file);
}
