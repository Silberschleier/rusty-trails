#include <iostream>
#include "dnghost.h"


#include "dng_auto_ptr.h"
#include "dng_preview.h"
#include "dng_date_time.h"

#include "dng_negative.h"
#include "dng_xmp_sdk.h"
#include "dng_render.h"
#include "dng_image_writer.h"
#include "dng_color_space.h"
#include "dng_exceptions.h"
#include "dng_tag_values.h"
#include "dng_xmp.h"
#include <dng_simple_image.h>


#include <stdexcept>

#include "rawConverter.h"

void raw2dng(std::string rawFilename, std::string outFilename, std::string dcpFilename, unsigned short *image_data) {
    RawConverter converter;
    converter.openRawFile(rawFilename);
    converter.buildNegative(dcpFilename, image_data);
    converter.renderImage();
    converter.renderPreviews();
    converter.writeDng(outFilename);
}

extern "C" void testcall_cpp(float value) {
    std::cout << "Hello, world from C++! Value passed: " << value << std::endl;
}

extern "C" void buildDNG(unsigned short * image_data, unsigned short int width, unsigned short int height) {
    std::cout << "Writing image of height " << height << " and width " << width << " to out.dng..." << std::endl;

    raw2dng("./Lapse_002/2018-02-24_001801_Canon EOS 450D_3708__MG_9903.CR2", "out.dng", "", image_data);

    /*dng_xmp_sdk::InitializeSDK();
    AutoPtr<dng_host> m_host;
    AutoPtr<dng_negative> m_negative;

    m_host.Reset(dynamic_cast<dng_host*>(new DngHost()));
    m_host->SetSaveDNGVersion(dngVersion_SaveDefault);
    m_host->SetSaveLinearDNG(false);
    m_host->SetKeepOriginalFile(true);

    dng_rect bounds = dng_rect(height, width);
    auto *image = new dng_simple_image(bounds, 1, ttShort, m_host->Allocator());

    dng_pixel_buffer buffer; image->GetPixelBuffer(buffer);
    auto *imageBuffer = (unsigned short*) buffer.fData;

    std::cout << "Copying image_data" << std::endl;
    memcpy(imageBuffer, image_data, height * width * sizeof(unsigned short));

    std::cout << "Cast new image" << std::endl;
    AutoPtr<dng_image> castImage(dynamic_cast<dng_image*>(image));

    std::cout << "SetStage1Image" << std::endl;
    try {
        m_negative->SetStage1Image(castImage);
    } catch (const std::exception& e) {
        std::cout << "Caught exception: " << e.what() << std::endl;
    }


*/
    std::cout << "Hello, World!" << std::endl;
}
