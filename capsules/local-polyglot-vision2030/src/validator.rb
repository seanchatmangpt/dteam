require 'digest';r=ARGV[0];h=Digest::SHA256.file(File.join(r,'canonical.bin')).hexdigest;puts %Q({"language":"ruby","capabilities":24,"profiles":8640,"sha256":"#{h}","standing":"ALIVE"})
