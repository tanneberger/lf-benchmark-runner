{ naersk, lib}:

naersk.buildPackage {
  pname = "lf-benchmark-runner";
  version = "0.1.0";

  src = ./.;

  cargoSha256 = lib.fakeSha256;

  meta = with lib; {
    description = "Rust tool which allows the extraction of data from lingua-franca benchmarks.";
    homepage = "";
    license = with licenses; [ bsd2 ];
  };
}
