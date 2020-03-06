extern crate cmake;
use cmake::Config;
use std::env;

fn main()
{
    let dst_dngwriter = Config::new("dngwriter").build();
    //eprintln!("{}", dst_dngwriter.display());
    println!("cargo:rustc-link-search=native={}", dst_dngwriter.display());
    println!("cargo:rustc-link-search=native={}/build/libdng/", dst_dngwriter.display());
    println!("cargo:rustc-link-search=native={}/build/libdng/dng-sdk/", dst_dngwriter.display());
    println!("cargo:rustc-link-search=native={}/build/libdng/xmp-sdk/", dst_dngwriter.display());
    println!("cargo:rustc-link-search=native={}/build/libdng/md5/", dst_dngwriter.display());

    println!("cargo:rustc-link-lib=static=raw2dng");
    println!("cargo:rustc-link-lib=static=dng");
    println!("cargo:rustc-link-lib=static=dng-sdk");
    println!("cargo:rustc-link-lib=static=xmp-sdk");
    println!("cargo:rustc-link-lib=static=md5");


    let target  = env::var("TARGET").unwrap();
    if target.contains("apple")
    {
        println!("cargo:rustc-link-lib=dylib=c++");
    }
    else if target.contains("linux")
    {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=dylib=jpeg");
        println!("cargo:rustc-link-lib=dylib=expat");
        println!("cargo:rustc-link-lib=dylib=z");
        println!("cargo:rustc-link-lib=dylib=exiv2");
        println!("cargo:rustc-link-lib=dylib=raw");
    }
    else
    {
        unimplemented!();
    }
}
