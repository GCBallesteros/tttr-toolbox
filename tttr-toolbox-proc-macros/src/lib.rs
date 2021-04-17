extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Expr, Ident, bracketed, token, Token, Type};

struct PTUTagRead {
    header: Ident,
    ty: Type,
    key: Expr,
}

impl Parse for PTUTagRead {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let header: Ident = input.parse()?;
        let _paren: token::Bracket = bracketed!(content in input);
        let key: Expr = content.parse()?;
        input.parse::<Token![as]>()?;
        let ty: Type = input.parse()?;

        Ok(PTUTagRead {
			header,
            ty,
            key,
        })
    }
}

// example use
// read_ptu_tag!(header[SOME_VALUE] as Int8);
#[proc_macro]
pub fn read_ptu_tag(input: TokenStream) -> TokenStream {
    let PTUTagRead {
        header,
        ty,
        key,
    } = parse_macro_input!(input as PTUTagRead);


    let output = quote!{
        if let PTUTag::#ty(x) = #header
            .get(#key)
            .ok_or_else(|| Error::InvalidHeader(String::from(
                format!("Header is missing {}", #key),
            )))? {
                *x
        } else {
            return Err(Error::WrongEnumVariant);
        }
    };
    TokenStream::from(output)
}

#[proc_macro_attribute]
pub fn make_ptu_stream(args: TokenStream, item: TokenStream) -> TokenStream {
    //let name: Ident = Parse::parse(args).unwrap();

    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let stream_type = parse_macro_input!(args as syn::Ident);
    let stream_name = format_ident!("{}Stream", stream_type);

    let output = quote!{
        #[allow(non_camel_case_types)]
        pub struct #stream_name {
            // todo: make it just with a trait that implements readbuf
            source: BufReader<std::fs::File>,
            click_buffer: [u32; BUFFER_SIZE],
            num_records: usize,
            time_resolution: f64,
            photons_in_buffer: i32,
            click_count: usize,
            overflow_correction: u64,
        }

        impl #stream_name {
            pub fn new(ptu_file: &ptu::PTUFile) -> Result<Self, Error> {
                let header = &ptu_file.header;
                let number_of_records: i64 = read_ptu_tag!(header[TAG_NUM_RECORDS] as Int8);
                let data_offset: i64 = read_ptu_tag!(header["DataOffset"] as Int8);

                let mut buffered = BufReader::with_capacity(8*1024, std::fs::File::open(ptu_file.path.clone())?);
                buffered.seek(SeekFrom::Start(data_offset as u64))?;

                Ok(Self {
                    source: buffered,
                    click_buffer: [0; BUFFER_SIZE],
                    num_records: number_of_records as usize,
                    time_resolution: ptu_file.time_resolution()?,
                    photons_in_buffer: 0,
                    click_count: 0,
                    overflow_correction: 0,
                })
            }
        }

        impl TTTRStream for #stream_name {
            type RecordSize = u32;
            #[inline(always)]
            #input

            fn time_resolution(&self) -> f64 {self.time_resolution}
        }

        impl Iterator for #stream_name {
            type Item = TTTRRecord;

            #[inline(always)]
            fn next(&mut self) -> Option<Self::Item> {
                if self.photons_in_buffer == 0 {
                    let read_res = self.source.read_u32_into::<NativeEndian>(&mut self.click_buffer[..]);
                    if let Err(_x) = read_res {
                        if self.click_count < self.num_records {
                            println!("Missed {}", self.num_records - self.click_count);
                        }
                        return None
                    };
                    self.photons_in_buffer = BUFFER_SIZE as i32;
                }

                let current_photon = ((BUFFER_SIZE as i32) - self.photons_in_buffer) as usize;
                self.photons_in_buffer -= 1;
                self.click_count += 1;
                Some(self.parse_record(self.click_buffer[current_photon]))
            }
        }
    };
    output.into()
}

