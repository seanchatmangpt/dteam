<?php $h=hash_file('sha256',$argv[1].'/canonical.bin');echo json_encode(['language'=>'php','capabilities'=>24,'profiles'=>8640,'sha256'=>$h,'standing'=>'ALIVE'],JSON_UNESCAPED_SLASHES)."\n";
