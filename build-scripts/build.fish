cargo build --release 
upx target/release/thrushc --best
echo "The Thrush Compiler binary size: " && du -h target/release/thrushc
echo "The Thrush Compiler was compiled succesfully."