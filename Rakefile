LIB_RABBITMQ_LIB_PATH = ENV['LIB_RABBITMQ_LIB_PATH'] || '/Users/andrii/dev/rabbitmq-c/build/librabbitmq'
LIB_RABBITMQ_INC_PATH = ENV['LIB_RABBITMQ_INC_PATH'] || '/Users/andrii/dev/rabbitmq-c/librabbitmq'

desc 'generate bindings'
task :bindings do
  sh "./bindgen -builtins -l rabbitmq -I#{LIB_RABBITMQ_INC_PATH} -o src/rabbitmq-c.rs gen.h"
end

desc 'compile crate'
file 'librabbitmq-4e165f79-1.0.rlib' => ['src/rabbitmqc.rs', 'src/rabbitmq.rs'] do
  sh "rustc src/rabbitmq.rs"
end

desc 'build hello world'
file 'hello' => ['hello.rs', 'librabbitmq-4e165f79-1.0.rlib'] do |task|
  sh "rustc -L . -L #{LIB_RABBITMQ_LIB_PATH} hello.rs"
end


task :default => ['hello']
