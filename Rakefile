LIB_RABBITMQ_LIB_PATH = ENV['LIB_RABBITMQ_LIB_PATH']
LIB_RABBITMQ_INC_PATH = ENV['LIB_RABBITMQ_INC_PATH']
CRATE_FILE = 'librabbitmq-4e165f79-1.0.rlib'

def crate_file
  Dir['librabbitmq*.rlib'].first || 'librabbitmq'
end

desc 'generate bindings'
task :bindings do
  sh "./bindgen -builtins -l rabbitmq #{"-I#{LIB_RABBITMQ_INC_PATH}" if LIB_RABBITMQ_INC_PATH} -o src/rabbitmq-c.rs gen.h"
end

desc 'compile crate file'
file crate_file => ['src/rabbitmqc.rs', 'src/rabbitmq.rs'] do
  sh "rustc src/rabbitmq.rs"
end

desc 'builds crate'
task :build => [crate_file]

desc 'build hello world'
file 'hello' => ['hello.rs', crate_file] do |task|
  sh "rustc -L . #{"-L #{LIB_RABBITMQ_LIB_PATH}" if LIB_RABBITMQ_LIB_PATH} hello.rs"
end

task :default => ['build', 'hello']
