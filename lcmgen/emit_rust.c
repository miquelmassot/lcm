#include <ctype.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#include "lcmgen.h"

#define INDENT(n) (4*(n))

#define emit_start(n, ...) do { fprintf(f, "%*s", INDENT(n), ""); fprintf(f, __VA_ARGS__); } while (0)
#define emit_continue(...) do { fprintf(f, __VA_ARGS__); } while (0)
#define emit_end(...) do { fprintf(f, __VA_ARGS__); fprintf(f, "\n"); } while (0)
#define emit(n, ...) do { fprintf(f, "%*s", INDENT(n), ""); fprintf(f, __VA_ARGS__); fprintf(f, "\n"); } while (0)

void setup_rust_options(getopt_t *gopt)
{
  getopt_add_string (gopt, 0, "rust-path",    ".",      "Location for .rs files");
}

static char *dots_to_slashes(const char *s)
{
    char *p = strdup(s);
    for (char *t=p; *t!=0; t++)
        if (*t == '.')
            *t = G_DIR_SEPARATOR;
    return p;
}

static char *make_rust_type_name(const char *input)
{
  char* result = strdup(input);
  char* result_char = result;
  int capitalize_next_char = 1;
  for (const char* c = input; *c != 0; ++c) {
    if (*c == '_') {
      capitalize_next_char = 1;
    } else {
      if (capitalize_next_char) {
        capitalize_next_char = 0;
        *result_char = toupper(*c);
      } else {
        *result_char = *c;
      }
      ++result_char;
    }
  }
  *result_char = 0;
  return result;
}

static const char * dim_size_prefix(const char *dim_size) {
    char *eptr = NULL;
    long asdf = strtol(dim_size, &eptr, 0);
    (void) asdf;  // suppress compiler warnings
    if(*eptr == '\0')
        return "";
    else
        return "this->";
}

static int is_dim_size_fixed(const char* dim_size) {
    char *eptr = NULL;
    long asdf = strtol(dim_size, &eptr, 0);
    (void) asdf;  // suppress compiler warnings
    return (*eptr == '\0');
}

static const char *map_type_name(const char *t)
{
  if (!strcmp(t, "boolean"))
    return "bool";

  if (!strcmp(t, "string"))
    return "String";

  if (!strcmp(t, "byte"))
    return "u8";

  if (!strcmp(t, "int8_t"))
    return "i8";

  if (!strcmp(t, "int16_t"))
    return "i16";

  if (!strcmp(t, "int32_t"))
    return "i32";

  if (!strcmp(t, "int64_t"))
    return "i64";

  if (!strcmp(t, "uint8_t"))
    return "u8";

  if (!strcmp(t, "uint16_t"))
    return "u16";

  if (!strcmp(t, "uint32_t"))
    return "u32";

  if (!strcmp(t, "uint64_t"))
    return "u64";

  if (!strcmp(t, "float"))
    return "f32";

  if (!strcmp(t, "double"))
    return "f64";

  return t;
}

static void make_dirs_for_file(const char *path)
{
#ifdef WIN32
    char *dirname = g_path_get_dirname(path);
    g_mkdir_with_parents(dirname, 0755);
    g_free(dirname);
#else
    int len = strlen(path);
    for (int i = 0; i < len; i++) {
        if (path[i]=='/') {
            char *dirpath = (char *) malloc(i+1);
            strncpy(dirpath, path, i);
            dirpath[i]=0;

            mkdir(dirpath, 0755);
            free(dirpath);

            i++; // skip the '/'
        }
    }
#endif
}

static void emit_header_start(lcmgen_t *lcmgen, FILE *f, lcm_struct_t *ls)
{
      emit(0, "// GENERATED CODE - DO NOT EDIT");
      emit(0, "");
      emit(0, "use lcm::generic_array::{GenericArray, typenum};");
      emit(0, "use lcm;");
      emit(0, "use std::io::{Result, Write};");
      emit(0, "");
}

static void emit_struct_def(lcmgen_t *lcmgen, FILE *f, lcm_struct_t *ls)
{
    char *sn = ls->structname->shortname;
    char *sn_camel = make_rust_type_name(sn);

    emit(0, "#[derive(Default)]");
    emit(0, "pub struct %s {", sn_camel);
    for (unsigned int mind = 0; mind < g_ptr_array_size(ls->members); mind++) {
      lcm_member_t *lm = (lcm_member_t *) g_ptr_array_index(ls->members, mind);
      char *mn = lm->membername;
      const char *mapped_typename = map_type_name(lm->type->lctypename);

      int ndim = g_ptr_array_size(lm->dimensions);
      emit_start(1, "pub %s: ", mn);
      if (ndim == 0) {
        emit_continue("%s", mapped_typename);
      } else {
        if (lcm_is_constant_size_array(lm)) {
          for (unsigned int d = 0; d < ndim; d++)
            emit_continue("GenericArray<");
          emit_continue("%s", mapped_typename);
          for (unsigned int d = 0; d < ndim; d++) {
            lcm_dimension_t *ld = (lcm_dimension_t *) g_ptr_array_index(lm->dimensions, d);
            emit_continue(", typenum::U%s>", ld->size);
          }
        } else {
          for (unsigned int d = 0; d < ndim; d++)
            emit_continue("Vec<");
          emit_continue("%s", mapped_typename);
          for (unsigned int d = 0; d < ndim; d++)
            emit_continue(">");
        }
      }
      emit_end(",");
    }
    emit(0, "}");
    emit(0, "");
}

static void emit_impl_struct(lcmgen_t *lcmgen, FILE *f, lcm_struct_t *ls)
{
  char *sn = ls->structname->shortname;
  char *sn_camel = make_rust_type_name(sn);

  emit(0, "impl %s {", sn_camel);
  emit(1, "pub fn new() -> Self {");
  emit(2, "Default::default()");
  emit(1, "}");
  emit(0, "}");
  emit(0, "");
}

static void emit_impl_encode(lcmgen_t *lcmgen, FILE *f, lcm_struct_t *ls)
{
  char *sn = ls->structname->shortname;
  char *sn_rust_name = make_rust_type_name(sn);

  emit(0, "impl lcm::Encode for %s {", sn_rust_name);

  emit(1, "fn encode(&self, mut buffer: &mut Write) -> Result<()> {");
  for (unsigned int mind = 0; mind < g_ptr_array_size(ls->members); mind++) {
    lcm_member_t *lm = (lcm_member_t *) g_ptr_array_index(ls->members, mind);
    char *mn = lm->membername;
    emit(2, "self.%s.encode(&mut buffer)?;", mn);
  }
  emit(2, "Ok(())");
  emit(1, "}");

  emit(0, "");

  emit(1, "fn size(&self) -> usize {");
  emit(2, "let mut size = 0;");
  for (unsigned int mind = 0; mind < g_ptr_array_size(ls->members); mind++) {
    lcm_member_t *lm = (lcm_member_t *) g_ptr_array_index(ls->members, mind);
    char *mn = lm->membername;
    emit(2, "size += self.%s.size();", mn);
  }
  emit(2, "size");
  emit(1, "}");

  emit(0, "}");
  emit(0, "");

  free(sn_rust_name);
}

static void emit_impl_message(lcmgen_t *lcmgen, FILE *f, lcm_struct_t *ls)
{
  char *sn = ls->structname->shortname;
  char *sn_camel = make_rust_type_name(sn);

  emit(0, "impl lcm::Message for %s {", sn_camel);

  emit(1,     "fn hash(&self) -> i64 {");
  emit(2,         "let hash = 0x%016"PRIx64";", ls->hash);
  emit(2,         "(hash << 1) + ((hash >> 63) & 1)");
  emit(1,     "}");

  emit(0, "}");
  emit(0, "");
}

int emit_rust(lcmgen_t *lcmgen)
{
  // iterate through all defined message types
  for (unsigned int i = 0; i < g_ptr_array_size(lcmgen->structs); i++) {
    lcm_struct_t *lr = (lcm_struct_t*) g_ptr_array_index(lcmgen->structs, i);

    const char *tn = lr->structname->lctypename;
    char *tn_ = dots_to_slashes(tn);

    // compute the target filename
    char *file_name = g_strdup_printf("%s%s%s.rs",
            getopt_get_string(lcmgen->gopt, "rust-path"),
            strlen(getopt_get_string(lcmgen->gopt, "rust-path")) > 0 ? G_DIR_SEPARATOR_S : "",
            tn_);

    // generate code if needed
    if (lcm_needs_generation(lcmgen, lr->lcmfile, file_name)) {
      make_dirs_for_file(file_name);

      FILE *f = fopen(file_name, "w");
      if (f == NULL)
        return -1;

      emit_header_start(lcmgen, f, lr);
      emit_struct_def(lcmgen, f, lr);
      emit_impl_struct(lcmgen, f, lr);
      emit_impl_encode(lcmgen, f, lr);
      emit_impl_message(lcmgen, f, lr);

      fclose(f);
    }

    g_free(file_name);
    free(tn_);
  }

  return 0;
}
